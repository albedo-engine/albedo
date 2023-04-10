use bytemuck::Pod;
use std::marker::PhantomData;
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

pub struct Buffer {
    inner: wgpu::Buffer,
    count: u64,
    bytes_per_element: u64,
}

impl Buffer {
    pub fn new_with_data<'a, T: Pod>(
        device: &wgpu::Device,
        contents: &[T],
        options: Option<BufferInitDescriptor<'a>>,
    ) -> Buffer {
        let options = options.unwrap_or_default();
        let bytes_per_element = std::mem::size_of::<T>() as u64;
        let inner = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: options.label,
            contents: bytemuck::cast_slice(contents),
            usage: options.usage,
        });
        Buffer {
            inner,
            count: contents.len() as u64,
            bytes_per_element,
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

pub struct TypedBuffer<T: Pod> {
    inner: Buffer,
    _content_type: PhantomData<T>,
}

impl<T: Pod> TypedBuffer<T> {
    pub fn new_with_data<'a>(
        device: &wgpu::Device,
        contents: &[T],
        options: Option<BufferInitDescriptor<'a>>,
    ) -> TypedBuffer<T> {
        let bytes_per_element = std::mem::size_of::<T>() as u64;
        TypedBuffer {
            inner: Buffer::new_with_data(device, contents, options),
            _content_type: PhantomData,
        }
    }
}

pub struct TypedUniformBuffer<T: Pod>(TypedBuffer<T>);

impl<T: Pod> TypedUniformBuffer<T> {
    pub fn new_with_data<'a>(
        device: &wgpu::Device,
        content: &T,
        options: Option<BufferInitDescriptor>,
    ) -> Self {
        let mut options = options.unwrap_or_default();
        options.usage = options.usage | wgpu::BufferUsages::UNIFORM;
        Self {
            0: TypedBuffer::new_with_data(device, &[*content], Some(options)),
        }
    }

    pub fn as_entire_binding(&self) -> wgpu::BindingResource {
        self.0.inner().as_entire_binding()
    }
}

impl<T: Pod> core::ops::Deref for TypedBuffer<T> {
    type Target = Buffer;

    fn deref(&self) -> &Buffer {
        &self.inner
    }
}

pub enum IndexBuffer {
    U16(TypedBuffer<u16>),
    U32(TypedBuffer<u32>),
}

impl IndexBuffer {
    pub fn new_with_data_16(
        device: &wgpu::Device,
        data: &[u16],
        options: Option<BufferInitDescriptor>,
    ) -> Self {
        let mut options = options.unwrap_or_default();
        options.usage = options.usage | wgpu::BufferUsages::INDEX;
        IndexBuffer::U16(TypedBuffer::new_with_data(device, data, Some(options)))
    }

    pub fn new_with_data_32(
        device: &wgpu::Device,
        data: &[u16],
        options: Option<BufferInitDescriptor>,
    ) -> Self {
        let mut options = options.unwrap_or_default();
        options.usage = options.usage | wgpu::BufferUsages::INDEX;
        IndexBuffer::U16(TypedBuffer::new_with_data(device, data, Some(options)))
    }

    pub fn inner(&self) -> &wgpu::Buffer {
        match self {
            IndexBuffer::U16(b) => b.inner(),
            IndexBuffer::U32(b) => b.inner(),
        }
    }
}

impl core::ops::Deref for IndexBuffer {
    type Target = Buffer;

    fn deref(&self) -> &Buffer {
        match self {
            IndexBuffer::U16(b) => b.deref(),
            IndexBuffer::U32(b) => b.deref(),
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

impl<T: Pod> From<TypedBuffer<T>> for Buffer {
    fn from(buffer: TypedBuffer<T>) -> Self {
        buffer.inner
    }
}

impl<'a> From<&'a Buffer> for &'a wgpu::Buffer {
    fn from(buffer: &'a Buffer) -> Self {
        buffer.inner()
    }
}

impl<'a, T: bytemuck::Pod> From<&'a TypedBuffer<T>> for &'a wgpu::Buffer {
    fn from(buffer: &'a TypedBuffer<T>) -> Self {
        (*buffer).inner()
    }
}

impl<'a, T: bytemuck::Pod> From<&'a GPUBuffer<T>> for &'a wgpu::Buffer {
    fn from(buffer: &'a GPUBuffer<T>) -> Self {
        buffer.inner()
    }
}
