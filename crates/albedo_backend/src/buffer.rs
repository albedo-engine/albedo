use bytemuck::Pod;
use std::marker::PhantomData;
use wgpu::util::DeviceExt;

use crate::DynSize;

#[derive(Clone, Copy)]
struct BufferOptions {
    pub count: u64,
    pub usage: wgpu::BufferUsages,
}

impl BufferOptions {
    pub fn new() -> Self {
        BufferOptions {
            ..Default::default()
        }
    }
}

impl Default for BufferOptions {
    fn default() -> Self {
        Self {
            count: Default::default(),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BufferInitDescriptor<'a, T> {
    /// Debug label of a buffer. This will show up in graphics debuggers for easy identification.
    pub label: wgpu::Label<'a>,
    /// Contents of a buffer on creation.
    pub contents: &'a [T],
    /// Usages of a buffer. If the buffer is used in any way that isn't specified here, the operation
    /// will panic.
    pub usage: wgpu::BufferUsages,
}

pub struct Buffer<T = crate::DynSize> {
    inner: wgpu::Buffer,
    count: u64,
    bytes_per_element: u64,
    _content_type: PhantomData<T>,
}

impl<T: Pod> Buffer<T> {
    pub fn new_with_data<'a>(
        device: &wgpu::Device,
        options: BufferInitDescriptor<'a, T>,
    ) -> Buffer<T> {
        let bytes_per_element = std::mem::size_of::<T>() as u64;
        let inner = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: options.label,
            contents: bytemuck::cast_slice(options.contents),
            usage: options.usage,
        });
        Buffer {
            inner,
            count: options.contents.len() as u64,
            bytes_per_element,
            _content_type: PhantomData,
        }
    }

    pub fn count(&self) -> u64 {
        self.count
    }

    pub fn bytes_per_element(&self) -> u64 {
        self.bytes_per_element
    }

    pub fn inner(&self) -> &wgpu::Buffer {
        &self.inner
    }
}

impl Buffer<DynSize> {
    pub fn new_with_data<'a, T: Pod>(
        device: &wgpu::Device,
        options: BufferInitDescriptor<'a, T>,
    ) -> Buffer<()> {
        let bytes_per_element = std::mem::size_of::<T>() as u64;
        let inner = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: options.label,
            contents: bytemuck::cast_slice(options.contents),
            usage: options.usage,
        });
        Buffer {
            inner,
            count: options.contents.len() as u64,
            bytes_per_element,
            _content_type: PhantomData,
        }
    }
}

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

impl<T: Pod> From<Buffer<T>> for Buffer<DynSize> {
    fn from(buffer: Buffer<T>) -> Self {
        Buffer {
            inner: buffer.inner,
            count: buffer.count,
            bytes_per_element: buffer.bytes_per_element,
            _content_type: PhantomData,
        }
    }
}

impl<'a, T: bytemuck::Pod> From<&'a Buffer<T>> for &'a wgpu::Buffer {
    fn from(buffer: &'a Buffer<T>) -> Self {
        buffer.inner()
    }
}

impl<'a, T: bytemuck::Pod> From<&'a GPUBuffer<T>> for &'a wgpu::Buffer {
    fn from(buffer: &'a GPUBuffer<T>) -> Self {
        buffer.inner()
    }
}
