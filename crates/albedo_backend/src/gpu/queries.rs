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
    /// Create a new set of options with defaults and `max_count`.
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
    /// Per frame query set.
    /// It's possible to share a single set for all frames. However,
    /// it looks like the current wgpu Metal implementation doesn't like that.
    /// @todo: Attempt to share the query set for all frames
    set: wgpu::QuerySet,
    /// CPU buffer for read back
    staging_buffer: wgpu::Buffer,
    /// Byte offset in the destination GPU buffer
    buffer_offset: u64,
    /// `true` if the frame is pending, `false` otherwise
    pending: Arc<AtomicBool>,
    /// Current query index
    query_index: u32,
}

impl PerFrame {
    pub fn write(&mut self, encoder: &mut wgpu::CommandEncoder) {
        encoder.write_timestamp(&self.set, self.query_index);
        self.query_index += 1;
    }
}

/// Set of queries to profile GPU commands.
/// @todo: Make thread safe
pub struct Queries {
    resolve_buffer: wgpu::Buffer,
    timestamp_count: u32,

    free_frames: Vec<PerFrame>,
    used_frames: Vec<PerFrame>,
    current_frame: Option<PerFrame>,

    values: Vec<f64>,
    labels: Vec<String>,
    resolved_queries_count: u32,
}

fn align_resolve_size(unaligned: u64) -> u64 {
    const POW2_ALIGNENT: u64 = QUERY_RESOLVE_BUFFER_ALIGNMENT - 1;
    (unaligned + POW2_ALIGNENT) & !(POW2_ALIGNENT)
}

fn queries_bytes(timestamp_count: u32) -> u64 {
    timestamp_count as u64 * wgpu::QUERY_SIZE as u64
}

/// # Example
///
/// ```
/// let queries = Queries::new(QueriesOptions::new(1));
///
/// let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
/// queries.start_frame(queue.get_timestamp_period());
///
/// queries.start("pass", &mut encoder);
/// // Dispatch compute passes or start render pass.
/// queries.end();
///
/// queue.submit(encoder.finish());
///
/// queries.end_frame(queue.get_timestamp_period());
/// ```
impl Queries {
    pub fn new(device: &wgpu::Device, opts: QueriesOptions) -> Self {
        let timestamp_count = opts.max_count * 2;
        let bytes_per_query_set: u64 = wgpu::QUERY_SIZE as u64 * timestamp_count as u64;

        let mut free_frames: Vec<PerFrame> = Vec::with_capacity(opts.max_frames_in_flight as usize);
        // Since frames are popped, reverse push to always get a null offset for the first frame.
        for i in (0..opts.max_frames_in_flight).rev() {
            free_frames.push(PerFrame {
                set: device.create_query_set(&wgpu::QuerySetDescriptor {
                    label: Some("Timestamp query set"),
                    count: timestamp_count,
                    ty: wgpu::QueryType::Timestamp,
                }),
                staging_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Query staging buffer"),
                    size: bytes_per_query_set,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                    mapped_at_creation: false,
                }),
                buffer_offset: align_resolve_size(i as u64 * bytes_per_query_set),
                pending: Arc::new(AtomicBool::new(false)),
                query_index: 0,
            });
        }
        Queries {
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
            resolved_queries_count: 0,
        }
    }

    /// Should be called at the beginning of a frame.
    ///
    /// `period` must be set to `queue.get_timestamp_period()`.
    pub fn start_frame(&mut self, period: f32) {
        if self.free_frames.len() == 0 {
            // It would be possible to unmap the newest buffer as well. On Metal,
            // doing so didn't bring anything compared to dropping frames.
            return;
        }

        let mut frame = self.free_frames.pop().unwrap();
        frame.query_index = 0;
        frame
            .pending
            .store(true, std::sync::atomic::Ordering::Relaxed);

        self.current_frame = Some(frame);

        self.try_read_state(period);
    }

    /// Start a new timer.
    ///
    /// This method must be followed by a called to [end].
    ///
    /// # Examples
    ///
    /// ```
    /// let queries = Queries::new(QueriesOptions::new(1));
    /// queries.start("my-query", encoder);
    /// // Do something with the encoder
    /// queries.end();
    /// ```
    pub fn start<T: ToString>(&mut self, label: T, encoder: &mut wgpu::CommandEncoder) {
        if let Some(frame) = self.current_frame.as_mut() {
            if frame.query_index == self.timestamp_count {
                panic!("maximum querie enties reached");
            }

            let index = frame.query_index / 2;
            self.labels[index as usize] = label.to_string();

            frame.write(encoder);
        }
    }

    /// End the timer started with [start].
    pub fn end(&mut self, encoder: &mut wgpu::CommandEncoder) {
        if let Some(frame) = self.current_frame.as_mut() {
            frame.write(encoder);
        }
    }

    /// Resolve queries added since the beginning of the frame.
    pub fn resolve(&self, encoder: &mut wgpu::CommandEncoder) {
        let Some(frame) = self.current_frame.as_ref() else {
            return;
        };

        encoder.resolve_query_set(
            &frame.set,
            0..frame.query_index as u32,
            &self.resolve_buffer,
            frame.buffer_offset,
        );
        encoder.copy_buffer_to_buffer(
            &self.resolve_buffer,
            frame.buffer_offset,
            &frame.staging_buffer,
            0,
            queries_bytes(self.timestamp_count),
        );
    }

    /// Should be called once the frame is finished and the encoder is dropped.
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

    /// Latest resolved values.
    ///
    /// Indexing is kept in the order of calls to [start].
    pub fn values(&self) -> &[f64] {
        let count = self.resolved_queries_count / 2;
        &self.values[0..count as usize]
    }

    /// Latest resolved label, one for each value.
    ///
    /// Indexing is kept in the order of calls to [start].
    pub fn labels(&self) -> &[String] {
        let count = self.resolved_queries_count / 2;
        &self.labels[0..count as usize]
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
            let count = frame.query_index;
            let bytes = count as u64 * wgpu::QUERY_SIZE as u64;
            let view = frame.staging_buffer.slice(..bytes).get_mapped_range();
            let timestamps: &[u64] = bytemuck::cast_slice(&view);

            const NS_TO_MS: f64 = 1.0 / 1000000.0;
            for i in (0..timestamps.len()).step_by(2) {
                let delta = timestamps[i + 1].wrapping_sub(timestamps[i]);
                let delta = delta as f64 * period as f64 * NS_TO_MS;

                let index = i / 2;
                self.values[index] = delta;
            }
            self.resolved_queries_count = count;
        }

        frame.staging_buffer.unmap();
        self.free_frames.push(frame);
    }
}
