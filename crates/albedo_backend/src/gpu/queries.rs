use std::sync::{atomic::AtomicBool, Arc};

struct PerFrame {
    staging_buffer: wgpu::Buffer,
    buffer_offset: u64,
    query_offset: u32,
    pending: Arc<AtomicBool>,
}

pub struct Queries {
    set: wgpu::QuerySet,
    resolve_buffer: wgpu::Buffer,
    timestamp_count: u32,
    next: u32,

    free_frames: Vec<PerFrame>,
    used_frames: Vec<PerFrame>,

    accumulation: usize,
    accumulation_count: usize,
    accumulated_values: Vec<f64>,

    skip: bool,

    values: Vec<f64>,
    labels: Vec<String>,
}

impl Queries {
    const MAX_FRAMES_IN_FLIGHT: u8 = 3;

    pub fn new(device: &wgpu::Device, count: u32) -> Self {
        let timestamp_count = count * 2;

        let staging_bytes = wgpu::QUERY_SIZE as u64 * timestamp_count as u64;
        let mut free_frames = Vec::with_capacity(Self::MAX_FRAMES_IN_FLIGHT as usize);
        for i in 0..Self::MAX_FRAMES_IN_FLIGHT as usize {
            let buffer_offset = {
                let unaligned =
                    i as u64 * std::mem::size_of::<u64>() as u64 * timestamp_count as u64;
                (unaligned + 255) & !(255)
            };
            let query_offset = i as u32 * timestamp_count;

            free_frames.push(PerFrame {
                staging_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("query dest buffer"),
                    size: staging_bytes,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                    mapped_at_creation: false,
                }),
                buffer_offset,
                query_offset,
                pending: Arc::new(AtomicBool::new(false)),
            });
        }
        // let size: u64 =
        //     Self::MAX_FRAMES_IN_FLIGHT as u64 * wgpu::QUERY_SIZE as u64 * timestamp_count as u64;
        Queries {
            set: device.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("Timestamp query set"),
                count: timestamp_count * Self::MAX_FRAMES_IN_FLIGHT as u32,
                ty: wgpu::QueryType::Timestamp,
            }),
            resolve_buffer: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("query resolve buffer"),
                size: Self::MAX_FRAMES_IN_FLIGHT as u64 * timestamp_count as u64 * 256,
                usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::QUERY_RESOLVE,
                mapped_at_creation: false,
            }),
            timestamp_count,
            next: 0,
            accumulation: 1,
            accumulation_count: 10,
            accumulated_values: vec![0.0; count as usize],

            free_frames,
            used_frames: Vec::with_capacity(Self::MAX_FRAMES_IN_FLIGHT as usize),
            skip: false,
            values: vec![0.0; count as usize],
            labels: (0..count).map(|_| String::new()).collect(),
        }
    }

    pub fn start_frame(&mut self) {
        self.next = 0;

        if self.free_frames.len() == 0 {
            self.skip = true;
            return;
        }

        self.skip = false;
        let frame = self.free_frames.pop().unwrap();
        frame
            .pending
            .store(true, std::sync::atomic::Ordering::Relaxed);

        self.used_frames.push(frame);
    }

    pub fn start<T: ToString>(&mut self, label: T, encoder: &mut wgpu::CommandEncoder) {
        if self.skip {
            return;
        }
        if self.next == self.timestamp_count {
            panic!("maximum querie enties reached");
        }
        self.labels[(self.next / 2) as usize] = label.to_string();
        self.write_timestamp(encoder);
    }

    pub fn end(&mut self, encoder: &mut wgpu::CommandEncoder) {
        if self.skip {
            return;
        }
        self.write_timestamp(encoder);
    }

    pub fn finish(&self, encoder: &mut wgpu::CommandEncoder) {
        if self.skip {
            return;
        }
        let frame = self.used_frames.last().unwrap();
        let buffer_size = self.bytes_per_set();
        encoder.resolve_query_set(
            &self.set,
            frame.query_offset..frame.query_offset + self.timestamp_count,
            &self.resolve_buffer,
            frame.buffer_offset,
        );
        encoder.copy_buffer_to_buffer(
            &self.resolve_buffer,
            frame.buffer_offset,
            &frame.staging_buffer,
            0,
            buffer_size,
        );
    }

    pub fn end_frame(&mut self, period: f32) {
        if !self.skip {
            let frame = self.used_frames.last().unwrap();

            let buffer_slice = frame.staging_buffer.slice(..);
            let pending = frame.pending.clone();
            buffer_slice.map_async(wgpu::MapMode::Read, move |state| match state {
                Ok(_) => {
                    pending.store(false, std::sync::atomic::Ordering::Relaxed);
                }
                _ => (),
            });
        }

        self.try_read_state(period);
    }

    pub fn values(&self) -> &[f64] {
        let count = (self.next / 2) as usize;
        &self.values[0..count]
    }

    pub fn labels(&self) -> &[String] {
        let count = (self.next / 2) as usize;
        &self.labels[0..count]
    }

    pub fn count(&self) -> u32 {
        self.timestamp_count / 2
    }

    fn bytes_per_set(&self) -> u64 {
        self.timestamp_count as u64 * wgpu::QUERY_SIZE as u64
    }

    fn write_timestamp(&mut self, encoder: &mut wgpu::CommandEncoder) {
        let frame = self.used_frames.last().unwrap();
        let query_index = frame.query_offset + self.next;
        encoder.write_timestamp(&self.set, query_index);

        self.next += 1;
    }

    fn try_read_state(&mut self, period: f32) {
        let Some(frame) = self.used_frames.first() else {
            return;
        };
        if frame.pending.load(std::sync::atomic::Ordering::Relaxed) {
            return;
        }

        const NS_TO_MS: f64 = 1.0 / 1000000.0;

        let frame = self.used_frames.remove(0);
        {
            let view = frame
                .staging_buffer
                .slice(..self.bytes_per_set())
                .get_mapped_range();
            let timestamps: &[u64] = bytemuck::cast_slice(&view);

            if self.accumulation == self.accumulation_count {
                self.accumulated_values.fill_with(|| 0.0);
                self.accumulation = 1;
            } else {
                self.accumulation += 1;
            }

            for i in (0..timestamps.len()).step_by(2) {
                let delta = timestamps[i + 1].wrapping_sub(timestamps[i]);
                let delta = delta as f64 * period as f64 * NS_TO_MS;

                let index = i / 2;
                self.accumulated_values[index] += delta;
                self.values[index] = self.accumulated_values[index] / self.accumulation as f64;
            }
        }

        frame.staging_buffer.unmap();
        self.free_frames.push(frame);
    }
}
