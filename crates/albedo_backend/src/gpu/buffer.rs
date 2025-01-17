use bytemuck::Pod;
use std::{convert::TryFrom, marker::PhantomData, ops::RangeBounds};
use wgpu::util::DeviceExt;

use crate::mesh::IndexData;

// @todo: Add a buffer builder.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReadBufferOptions {
    pub src_offset: u64,
    pub dst_offset: u64,
    pub count: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

pub struct DynBuffer {
    inner: wgpu::Buffer,
    byte_size: u64,
}

impl DynBuffer {
    pub fn new(
        device: &wgpu::Device,
        byte_size: u64,
        count: u64,
        options: Option<BufferInitDescriptor>,
    ) -> Self {
        let options = options.unwrap_or_default();
        let inner = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: byte_size * count as u64,
            usage: options.usage,
            mapped_at_creation: false,
        });
        Self { inner, byte_size }
    }

    pub fn new_with_data(
        device: &wgpu::Device,
        content: &[u8],
        count: u64,
        options: Option<BufferInitDescriptor>,
    ) -> Self {
        let options: BufferInitDescriptor = options.unwrap_or_default();
        let byte_size = content.len() as u64 / count;
        let inner = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: options.label,
            contents: bytemuck::cast_slice(content),
            usage: options.usage,
        });
        Self { inner, byte_size }
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

    pub fn usage(&self) -> wgpu::BufferUsages {
        self.inner.usage()
    }

    pub fn slice<S>(&self, bounds: S) -> wgpu::BufferSlice
    where
        S: RangeBounds<wgpu::BufferAddress>,
    {
        self.inner.slice(bounds)
    }

    pub fn as_entire_binding(&self) -> wgpu::BindingResource {
        self.inner.as_entire_binding()
    }

    pub fn as_sub_binding(&self, element_count: u64) -> wgpu::BindingResource {
        wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer: self.inner(),
            offset: 0,
            size: wgpu::BufferSize::new(element_count * self.byte_size),
        })
    }
}

pub struct Buffer<T: Pod> {
    inner: DynBuffer,
    _content_type: PhantomData<T>,
}

impl<T: Pod> Buffer<T> {
    pub fn dummy(device: &wgpu::Device, options: Option<BufferInitDescriptor>) -> Self {
        Self::new(device, 1, options)
    }

    pub fn dummy_storage(device: &wgpu::Device) -> Self {
        Self::new_storage(device, 1, None)
    }

    pub fn new(device: &wgpu::Device, count: u64, options: Option<BufferInitDescriptor>) -> Self {
        let byte_size = std::mem::size_of::<T>() as u64;
        let inner = DynBuffer::new(device, byte_size, count, options);
        Self {
            inner,
            _content_type: PhantomData,
        }
    }

    pub fn new_with_data(
        device: &wgpu::Device,
        content: &[T],
        options: Option<BufferInitDescriptor>,
    ) -> Self {
        let inner = DynBuffer::new_with_data(
            device,
            bytemuck::cast_slice(content),
            content.len() as u64,
            options,
        );
        Buffer {
            inner,
            _content_type: PhantomData,
        }
    }

    pub fn new_storage(
        device: &wgpu::Device,
        count: u64,
        options: Option<BufferInitDescriptor>,
    ) -> Self {
        let mut options = options.unwrap_or_default();
        options.usage = options.usage | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
        Buffer::new(device, count, Some(options))
    }

    pub fn new_uniform(
        device: &wgpu::Device,
        count: u64,
        options: Option<BufferInitDescriptor>,
    ) -> Self {
        let mut options = options.unwrap_or_default();
        options.usage = options.usage | wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST;
        Buffer::new(device, count, Some(options))
    }

    pub fn new_storage_with_data(
        device: &wgpu::Device,
        content: &[T],
        options: Option<BufferInitDescriptor>,
    ) -> Self {
        let mut options = options.unwrap_or_default();
        options.usage = options.usage | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
        Buffer::new_with_data(device, content, Some(options))
    }

    pub fn new_vertex_with_data(
        device: &wgpu::Device,
        content: &[T],
        options: Option<BufferInitDescriptor>,
    ) -> Self {
        let mut options = options.unwrap_or_default();
        options.usage = options.usage | wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST;
        Buffer::new_with_data(device, content, Some(options))
    }

    pub fn update(&mut self, queue: &wgpu::Queue, content: &[T]) {
        let slice = bytemuck::cast_slice(content);
        queue.write_buffer(&self.inner, 0, slice);
    }

    pub fn count(&self) -> u64 {
        self.inner.count()
    }

    pub fn usage(&self) -> wgpu::BufferUsages {
        self.inner.usage()
    }

    pub fn inner(&self) -> &wgpu::Buffer {
        &self.inner.inner()
    }

    pub fn as_entire_binding(&self) -> wgpu::BindingResource {
        self.inner.as_entire_binding()
    }

    pub fn as_sub_binding(&self, element_count: u64) -> wgpu::BindingResource {
        self.inner.as_sub_binding(element_count)
    }

    pub fn as_uniform_slice<'a>(&'a self) -> Result<UniformBufferSlice<'a, T>, ()> {
        if self.usage().contains(wgpu::BufferUsages::UNIFORM) {
            Ok(UniformBufferSlice::new(self))
        } else {
            Err(())
        }
    }

    pub fn as_storage_slice<'a>(&'a self) -> Result<StorageBufferSlice<'a, T>, ()> {
        if self.usage().contains(wgpu::BufferUsages::STORAGE) {
            Ok(StorageBufferSlice::new(self))
        } else {
            Err(())
        }
    }
}

pub enum IndexBuffer {
    U16(Buffer<u16>),
    U32(Buffer<u32>),
}

impl IndexBuffer {
    pub fn new_with_data(
        device: &wgpu::Device,
        indices: &crate::mesh::IndexData,
        options: Option<BufferInitDescriptor>,
    ) -> Self {
        match indices {
            IndexData::U16(data) => Self::new_with_data_16(device, data, options),
            IndexData::U32(data) => Self::new_with_data_32(device, data, options),
        }
    }

    pub fn new_with_data_16<'a>(
        device: &wgpu::Device,
        content: &[u16],
        options: Option<BufferInitDescriptor>,
    ) -> Self {
        let mut options = options.unwrap_or_default();
        options.usage = options.usage | wgpu::BufferUsages::INDEX;
        IndexBuffer::U16(Buffer::new_with_data(device, content, Some(options)))
    }

    pub fn new_with_data_32<'a>(
        device: &wgpu::Device,
        content: &[u32],
        options: Option<BufferInitDescriptor>,
    ) -> Self {
        let mut options = options.unwrap_or_default();
        options.usage = options.usage | wgpu::BufferUsages::INDEX;
        IndexBuffer::U32(Buffer::new_with_data(device, content, Some(options)))
    }

    pub fn inner(&self) -> &wgpu::Buffer {
        match self {
            IndexBuffer::U16(b) => b.inner(),
            IndexBuffer::U32(b) => b.inner(),
        }
    }

    pub fn count(&self) -> u64 {
        match self {
            IndexBuffer::U16(b) => b.count(),
            IndexBuffer::U32(b) => b.count(),
        }
    }

    pub fn as_entire_binding(&self) -> wgpu::BindingResource {
        match self {
            IndexBuffer::U16(b) => b.as_entire_binding(),
            IndexBuffer::U32(b) => b.as_entire_binding(),
        }
    }
}

#[derive(Copy, Clone)]
pub struct UniformBufferSlice<'a, T: Pod>(&'a Buffer<T>);

impl<'a, T: Pod> UniformBufferSlice<'a, T> {
    pub fn new(buffer: &'a Buffer<T>) -> Self {
        Self { 0: buffer }
    }
}

#[derive(Copy, Clone)]
pub struct StorageBufferSlice<'a, T: Pod>(&'a Buffer<T>);

impl<'a, T: Pod> StorageBufferSlice<'a, T> {
    pub fn new(buffer: &'a Buffer<T>) -> Self {
        Self { 0: buffer }
    }
}

// Traits //

impl<'a, T: Pod> std::ops::Deref for UniformBufferSlice<'a, T> {
    type Target = Buffer<T>;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a, T: Pod> TryFrom<&'a Buffer<T>> for UniformBufferSlice<'a, T> {
    type Error = ();
    fn try_from(buffer: &'a Buffer<T>) -> Result<Self, Self::Error> {
        buffer.as_uniform_slice()
    }
}

impl<'a, T: Pod> std::ops::Deref for StorageBufferSlice<'a, T> {
    type Target = Buffer<T>;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a, T: Pod> TryFrom<&'a Buffer<T>> for StorageBufferSlice<'a, T> {
    type Error = ();
    fn try_from(buffer: &'a Buffer<T>) -> Result<Self, Self::Error> {
        buffer.as_storage_slice()
    }
}

impl std::ops::Deref for DynBuffer {
    type Target = wgpu::Buffer;

    fn deref(&self) -> &Self::Target {
        self.inner()
    }
}
