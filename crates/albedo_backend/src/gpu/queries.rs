use std::sync::{atomic::AtomicBool, Arc};

use wgpu::QUERY_RESOLVE_BUFFER_ALIGNMENT;

/// Options to forward to [Queries::new].
#[derive(Clone, Copy)]
pub struct QueriesOptions {
    /// Maximum number of **queries**, not **timestamps**.
    pub max_count: u32,
    /// Maximum number of frames in-flight. This might need to be
    /// depending on the application performance.
    ///
    /// When this number is reached, incoming frames will be dropped.
    pub max_frames_in_flight: u8,
}

impl QueriesOptions {
    pub fn new(max_count: u32) -> Self {
        QueriesOptions {
            max_count,
            ..Default::default()
        }
    }
}

impl Default for QueriesOptions {
    fn default() -> Self {
        Self {
            max_count: 4,
            max_frames_in_flight: 5,
        }
    }
}

/// Per frame data
struct PerFrame {
    /// CPU buffer for read back
    staging_buffer: wgpu::Buffer,
    /// Byte offset in the destination GPU buffer
    buffer_offset: u64,
    /// Index offset of the first query in the query set
    query_offset: u32,
    /// `true` if the frame is pending, `false` otherwise
    pending: Arc<AtomicBool>,
    timestamp_count: usize,
}

impl PerFrame {
    pub fn query_range(&self) -> std::ops::Range<u32> {
        let count = self.timestamp_count as u32;
        self.query_offset..self.query_offset + count
    }
    pub fn write(&mut self, set: &wgpu::QuerySet, encoder: &mut wgpu::CommandEncoder) {
        let query_index = self.query_index();
        encoder.write_timestamp(set, query_index);
        self.timestamp_count += 1;
    }
    fn query_index(&self) -> u32 {
        self.query_offset + self.timestamp_count as u32
    }
}

/// Set of queries to profile GPU commands.
/// @todo: Make thread safe
pub struct Queries {
    set: wgpu::QuerySet,
    resolve_buffer: wgpu::Buffer,
    timestamp_count: u32,

    free_frames: Vec<PerFrame>,
    used_frames: Vec<PerFrame>,
    current_frame: Option<PerFrame>,

    values: Vec<f64>,
    labels: Vec<String>,
    last_resolved_count: usize,
}

fn align_resolve_size(unaligned: u64) -> u64 {
    const POW2_ALIGNENT: u64 = QUERY_RESOLVE_BUFFER_ALIGNMENT - 1;
    (unaligned + POW2_ALIGNENT) & !(POW2_ALIGNENT)
}

impl Queries {
    pub fn new(device: &wgpu::Device, opts: QueriesOptions) -> Self {
        let timestamp_count = opts.max_count * 2;
        let bytes_per_query_set: u64 = wgpu::QUERY_SIZE as u64 * timestamp_count as u64;

        let mut free_frames: Vec<PerFrame> = Vec::with_capacity(opts.max_frames_in_flight as usize);
        // Since frames are popped, reverse push to always get a null offset for the first frame.
        for i in (0..opts.max_frames_in_flight).rev() {
            free_frames.push(PerFrame {
                staging_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Query staging buffer"),
                    size: bytes_per_query_set,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                    mapped_at_creation: false,
                }),
                buffer_offset: align_resolve_size(i as u64 * bytes_per_query_set),
                query_offset: i as u32 * timestamp_count,
                pending: Arc::new(AtomicBool::new(false)),
                timestamp_count: 0,
            });
        }
        Queries {
            set: device.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("Timestamp query set"),
                count: timestamp_count * opts.max_frames_in_flight as u32,
                ty: wgpu::QueryType::Timestamp,
            }),
            resolve_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Query resolve buffer"),
                size: align_resolve_size(opts.max_frames_in_flight as u64 * bytes_per_query_set),
                usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::QUERY_RESOLVE,
                mapped_at_creation: false,
            }),

            timestamp_count,

            free_frames,
            used_frames: Vec::with_capacity(opts.max_frames_in_flight as usize),
            current_frame: None,

            values: vec![0.0; opts.max_count as usize],
            labels: (0..opts.max_count).map(|_| String::new()).collect(),
            last_resolved_count: 0,
        }
    }

    pub fn start_frame(&mut self, period: f32) {
        if self.free_frames.len() == 0 {
            // let new_frame = self.used_frames.pop().unwrap();
            // new_frame.staging_buffer.unmap();
            // self.free_frames.push(new_frame);
            return;
        }

        let mut frame = self.free_frames.pop().unwrap();
        frame.timestamp_count = 0;
        frame
            .pending
            .store(true, std::sync::atomic::Ordering::Relaxed);

        self.current_frame = Some(frame);

        self.try_read_state(period);
    }

    pub fn start<T: ToString>(&mut self, label: T, encoder: &mut wgpu::CommandEncoder) {
        if let Some(frame) = self.current_frame.as_mut() {
            if frame.timestamp_count == self.timestamp_count as usize {
                panic!("maximum querie enties reached");
            }
            let index = frame.timestamp_count / 2;
            self.labels[index] = label.to_string();

            frame.write(&self.set, encoder);
        }
    }

    pub fn end(&mut self, encoder: &mut wgpu::CommandEncoder) {
        if let Some(frame) = self.current_frame.as_mut() {
            frame.write(&self.set, encoder);
        }
    }

    pub fn finish(&self, encoder: &mut wgpu::CommandEncoder) {
        let Some(frame) = self.current_frame.as_ref() else {
            return;
        };
        let buffer_size: u64 = self.bytes_per_set();
        let range = frame.query_range();

        encoder.resolve_query_set(&self.set, range, &self.resolve_buffer, frame.buffer_offset);
        encoder.copy_buffer_to_buffer(
            &self.resolve_buffer,
            frame.buffer_offset,
            &frame.staging_buffer,
            0,
            buffer_size,
        );
    }

    pub fn end_frame(&mut self, period: f32) {
        let current_frame = self.current_frame.take();
        if let Some(frame) = current_frame {
            let buffer_slice = frame.staging_buffer.slice(..);
            let pending = frame.pending.clone();
            buffer_slice.map_async(wgpu::MapMode::Read, move |_| {
                pending.store(false, std::sync::atomic::Ordering::Relaxed);
            });
            self.used_frames.push(frame);
        }

        self.try_read_state(period);
    }

    pub fn values(&self) -> &[f64] {
        let count = self.last_resolved_count / 2;
        &self.values[0..count]
    }

    pub fn labels(&self) -> &[String] {
        let count = self.last_resolved_count / 2;
        &self.labels[0..count]
    }

    pub fn count(&self) -> u32 {
        self.timestamp_count / 2
    }

    fn bytes_per_set(&self) -> u64 {
        self.timestamp_count as u64 * wgpu::QUERY_SIZE as u64
    }

    fn try_read_state(&mut self, period: f32) {
        let Some(frame) = self.used_frames.first() else {
            return;
        };
        if frame.pending.load(std::sync::atomic::Ordering::Relaxed) {
            return;
        }

        let frame = self.used_frames.remove(0);
        {
            let bytes = frame.timestamp_count as u64 * wgpu::QUERY_SIZE as u64;
            let view = frame.staging_buffer.slice(..bytes).get_mapped_range();
            let timestamps: &[u64] = bytemuck::cast_slice(&view);

            const NS_TO_MS: f64 = 1.0 / 1000000.0;
            for i in (0..timestamps.len()).step_by(2) {
                let delta = timestamps[i + 1].wrapping_sub(timestamps[i]);
                let delta = delta as f64 * period as f64 * NS_TO_MS;

                let index = i / 2;
                self.values[index] = delta;
            }
            self.last_resolved_count = frame.timestamp_count;
        }

        frame.staging_buffer.unmap();
        self.free_frames.push(frame);
    }
}
