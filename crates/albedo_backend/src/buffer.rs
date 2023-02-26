use std::marker::PhantomData;
use wgpu;

pub struct GPUBuffer<T> {
    gpu_buffer: wgpu::Buffer,
    count: usize,
    content_type: PhantomData<T>,
}

impl<T: bytemuck::Pod> GPUBuffer<T> {
    pub fn new(device: &wgpu::Device) -> Self {
        GPUBuffer::new_with_usage(
            device,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        )
    }

    pub fn new_with_usage_count(
        device: &wgpu::Device,
        usage: wgpu::BufferUsages,
        count: usize,
    ) -> Self {
        let byte_count = (std::mem::size_of::<T>() * count) as u64;

        let gpu_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: byte_count,
            usage,
            mapped_at_creation: false,
        });
        GPUBuffer {
            gpu_buffer,
            count,
            content_type: PhantomData,
        }
    }

    pub fn new_with_usage(device: &wgpu::Device, usage: wgpu::BufferUsages) -> Self {
        GPUBuffer::new_with_usage_count(device, usage, 0)
    }

    pub fn new_with_count(device: &wgpu::Device, count: usize) -> Self {
        GPUBuffer::new_with_usage_count(
            device,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            count,
        )
    }

    pub fn from_data(device: &wgpu::Device, content: &[T]) -> Self {
        let byte_count = (std::mem::size_of::<T>() * content.len()) as u64;
        let gpu_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: byte_count,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        GPUBuffer {
            gpu_buffer,
            count: content.len(),
            content_type: PhantomData,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, content: &[T]) {
        let slice = bytemuck::cast_slice(content);
        queue.write_buffer(&self.gpu_buffer, 0, slice);
    }

    pub fn as_entire_binding(&self) -> wgpu::BindingResource {
        self.gpu_buffer.as_entire_binding()
    }

    pub fn fits(&self, content: &[T]) -> bool {
        content.len() <= self.count
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn size(&self) -> wgpu::BufferSize {
        wgpu::BufferSize::new((std::mem::size_of::<T>() * self.count) as u64).unwrap()
    }

    pub fn inner(&self) -> &wgpu::Buffer {
        &self.gpu_buffer
    }
}

// @todo: refactor code with GPUBuffer.
pub struct UniformBuffer<T> {
    gpu_buffer: wgpu::Buffer,
    content_type: PhantomData<T>,
}

impl<T: bytemuck::Pod> UniformBuffer<T> {
    pub fn new(device: &wgpu::Device) -> Self {
        let gpu_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<T>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        UniformBuffer {
            gpu_buffer,
            content_type: PhantomData,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, content: &T) {
        queue.write_buffer(&self.gpu_buffer, 0, bytemuck::bytes_of(content));
    }

    pub fn as_entire_binding(&self) -> wgpu::BindingResource {
        self.gpu_buffer.as_entire_binding()
    }
}

// Traits //

impl<'a, T: bytemuck::Pod> From<&'a GPUBuffer<T>> for &'a wgpu::Buffer {
    fn from(buffer: &'a GPUBuffer<T>) -> Self {
        buffer.inner()
    }
}
