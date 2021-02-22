use std::marker::PhantomData;
use wgpu;
// @todo: migrate to gfx.
pub struct GPUBuffer<T> {
    gpu_buffer: wgpu::Buffer,
    content_type: PhantomData<T>,
}

impl<T> GPUBuffer<T> {

    pub fn new(device: &wgpu::Device) -> Self {
        let gpu_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 0,
            usage: wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });
        GPUBuffer {
            gpu_buffer,
            content_type: PhantomData
        }
    }

    pub fn from_data(device: &wgpu::Device, content: Vec<T>) -> Self {
        let gpu_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (std::mem::size_of::<T>() * content.len()) as u64,
            usage: wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });
        GPUBuffer {
            gpu_buffer,
            content_type: PhantomData
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, content: Vec<T>) {
        queue.write_buffer(&self.gpu_buffer, 0, bytemuck::bytes_of(&content));
    }

}
