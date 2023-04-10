use bytemuck::Pod;
use std::{marker::PhantomData, ops::Deref};
use wgpu::util::DeviceExt;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BufferInitDescriptor<'a> {
    /// Debug label of a buffer. This will show up in graphics debuggers for easy identification.
    pub label: wgpu::Label<'a>,
    /// Usages of a buffer. If the buffer is used in any way that isn't specified here, the operation
    /// will panic.
    pub usage: wgpu::BufferUsages,
}

impl<'a> BufferInitDescriptor<'a> {
    pub fn new(label: wgpu::Label<'a>, usage: wgpu::BufferUsages) -> Self {
        BufferInitDescriptor { label, usage }
    }
    pub fn with_label(label: wgpu::Label<'a>) -> Self {
        BufferInitDescriptor {
            label,
            usage: wgpu::BufferUsages::COPY_DST,
        }
    }
}

impl<'a> Default for BufferInitDescriptor<'a> {
    fn default() -> Self {
        BufferInitDescriptor::new(None, wgpu::BufferUsages::COPY_DST)
    }
}

pub struct Buffer<T: Pod = ()> {
    inner: wgpu::Buffer,
    byte_size: u64,
    _content_type: PhantomData<T>,
}

impl<T: Pod> Buffer<T> {
    pub fn with_data(
        device: &wgpu::Device,
        contents: &[u8],
        count: u64,
        options: Option<BufferInitDescriptor>,
    ) -> Buffer<()> {
        let options = options.unwrap_or_default();
        let byte_size = contents.len() as u64 / count;
        let inner = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: options.label,
            contents,
            usage: options.usage,
        });
        Buffer {
            inner,
            byte_size,
            _content_type: PhantomData,
        }
    }

    pub fn with_count(
        device: &wgpu::Device,
        byte_size: u64,
        count: u64,
        options: Option<BufferInitDescriptor>,
    ) -> Self {
        let options = options.unwrap_or_default();
        let inner = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: byte_size * count,
            usage: options.usage,
            mapped_at_creation: false,
        });
        Self {
            inner,
            byte_size,
            _content_type: PhantomData,
        }
    }

    pub fn sized_with_data<'a>(
        device: &wgpu::Device,
        contents: &[T],
        options: Option<BufferInitDescriptor<'a>>,
    ) -> Buffer<T> {
        let options = options.unwrap_or_default();
        let byte_size = std::mem::size_of::<T>() as u64;
        let inner = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: options.label,
            contents: bytemuck::cast_slice(contents),
            usage: options.usage,
        });
        Buffer {
            inner,
            byte_size,
            _content_type: PhantomData,
        }
    }

    pub fn to_dynamic(self) -> Buffer {
        Buffer {
            inner: self.inner,
            byte_size: self.byte_size,
            _content_type: PhantomData,
        }
    }

    pub fn count(&self) -> u64 {
        self.inner.size() / self.byte_size
    }

    pub fn byte_size(&self) -> u64 {
        self.byte_size
    }

    pub fn inner(&self) -> &wgpu::Buffer {
        &self.inner
    }

    pub fn as_entire_binding(&self) -> wgpu::BindingResource {
        self.inner.as_entire_binding()
    }
}

pub struct UniformBuffer<T: Pod>(Buffer<T>);

impl<T: Pod> UniformBuffer<T> {
    pub fn sized_with_data<'a>(
        device: &wgpu::Device,
        content: &T,
        options: Option<BufferInitDescriptor>,
    ) -> Self {
        let mut options = options.unwrap_or_default();
        options.usage = options.usage | wgpu::BufferUsages::UNIFORM;
        Self {
            0: Buffer::sized_with_data(device, &[*content], Some(options)),
        }
    }
}

impl<T: Pod> Deref for UniformBuffer<T> {
    type Target = Buffer<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct IndexBufer16(Buffer<u16>);

impl Deref for IndexBufer16 {
    type Target = Buffer<u16>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IndexBufer16 {
    pub fn with_data<'a>(
        device: &wgpu::Device,
        content: &[u16],
        options: Option<BufferInitDescriptor>,
    ) -> Self {
        let mut options = options.unwrap_or_default();
        options.usage = options.usage | wgpu::BufferUsages::INDEX;
        Self {
            0: Buffer::sized_with_data(device, content, Some(options)),
        }
    }
}

pub struct IndexBufer32(Buffer<u32>);

impl Deref for IndexBufer32 {
    type Target = Buffer<u32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IndexBufer32 {
    pub fn with_data<'a>(
        device: &wgpu::Device,
        content: &[u32],
        options: Option<BufferInitDescriptor>,
    ) -> Self {
        let mut options = options.unwrap_or_default();
        options.usage = options.usage | wgpu::BufferUsages::INDEX;
        Self {
            0: Buffer::sized_with_data(device, content, Some(options)),
        }
    }
}

pub enum IndexBuffer {
    U16(IndexBufer16),
    U32(IndexBufer32),
}

impl IndexBuffer {
    pub fn with_data_16<'a>(
        device: &wgpu::Device,
        content: &[u16],
        options: Option<BufferInitDescriptor>,
    ) -> Self {
        IndexBuffer::U16(IndexBufer16::with_data(device, content, options))
    }

    pub fn with_data_32<'a>(
        device: &wgpu::Device,
        content: &[u32],
        options: Option<BufferInitDescriptor>,
    ) -> Self {
        IndexBuffer::U32(IndexBufer32::with_data(device, content, options))
    }

    pub fn inner(&self) -> &wgpu::Buffer {
        match self {
            IndexBuffer::U16(b) => b.0.inner(),
            IndexBuffer::U32(b) => b.0.inner(),
        }
    }

    pub fn count(&self) -> u64 {
        match self {
            IndexBuffer::U16(b) => b.0.count(),
            IndexBuffer::U32(b) => b.0.count(),
        }
    }

    pub fn as_entire_binding(&self) -> wgpu::BindingResource {
        match self {
            IndexBuffer::U16(b) => b.as_entire_binding(),
            IndexBuffer::U32(b) => b.as_entire_binding(),
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

// Traits //

impl<'a> From<&'a Buffer> for &'a wgpu::Buffer {
    fn from(buffer: &'a Buffer) -> Self {
        buffer.inner()
    }
}

impl<'a, T: bytemuck::Pod> From<&'a GPUBuffer<T>> for &'a wgpu::Buffer {
    fn from(buffer: &'a GPUBuffer<T>) -> Self {
        buffer.inner()
    }
}
