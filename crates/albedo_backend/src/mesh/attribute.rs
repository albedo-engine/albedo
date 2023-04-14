use std::marker::PhantomData;

use bytemuck::Pod;

fn compute_stride(formats: &[wgpu::VertexFormat]) -> u64 {
    let mut stride = 0;
    for format in formats {
        stride += format.size();
    }
    stride
}

#[derive(Debug)]
pub enum AttributeData {
    Float32(Vec<f32>),
    Sint32(Vec<i32>),
    Uint32(Vec<u32>),
    Float32x2(Vec<[f32; 2]>),
    Sint32x2(Vec<[i32; 2]>),
    Uint32x2(Vec<[u32; 2]>),
    Float32x3(Vec<[f32; 3]>),
    Sint32x3(Vec<[i32; 3]>),
    Uint32x3(Vec<[u32; 3]>),
    Float32x4(Vec<[f32; 4]>),
    Sint32x4(Vec<[i32; 4]>),
    Uint32x4(Vec<[u32; 4]>),
    Sint16x2(Vec<[i16; 2]>),
    Snorm16x2(Vec<[i16; 2]>),
    Uint16x2(Vec<[u16; 2]>),
    Unorm16x2(Vec<[u16; 2]>),
    Sint16x4(Vec<[i16; 4]>),
    Snorm16x4(Vec<[i16; 4]>),
    Uint16x4(Vec<[u16; 4]>),
    Unorm16x4(Vec<[u16; 4]>),
    Sint8x2(Vec<[i8; 2]>),
    Snorm8x2(Vec<[i8; 2]>),
    Uint8x2(Vec<[u8; 2]>),
    Unorm8x2(Vec<[u8; 2]>),
    Sint8x4(Vec<[i8; 4]>),
    Snorm8x4(Vec<[i8; 4]>),
    Uint8x4(Vec<[u8; 4]>),
    Unorm8x4(Vec<[u8; 4]>),
}

impl From<&AttributeData> for wgpu::VertexFormat {
    fn from(values: &AttributeData) -> Self {
        match values {
            AttributeData::Float32(_) => wgpu::VertexFormat::Float32,
            AttributeData::Sint32(_) => wgpu::VertexFormat::Sint32,
            AttributeData::Uint32(_) => wgpu::VertexFormat::Uint32,
            AttributeData::Float32x2(_) => wgpu::VertexFormat::Float32x2,
            AttributeData::Sint32x2(_) => wgpu::VertexFormat::Sint32x2,
            AttributeData::Uint32x2(_) => wgpu::VertexFormat::Uint32x2,
            AttributeData::Float32x3(_) => wgpu::VertexFormat::Float32x3,
            AttributeData::Sint32x3(_) => wgpu::VertexFormat::Sint32x3,
            AttributeData::Uint32x3(_) => wgpu::VertexFormat::Uint32x3,
            AttributeData::Float32x4(_) => wgpu::VertexFormat::Float32x4,
            AttributeData::Sint32x4(_) => wgpu::VertexFormat::Sint32x4,
            AttributeData::Uint32x4(_) => wgpu::VertexFormat::Uint32x4,
            AttributeData::Sint16x2(_) => wgpu::VertexFormat::Sint16x2,
            AttributeData::Snorm16x2(_) => wgpu::VertexFormat::Snorm16x2,
            AttributeData::Uint16x2(_) => wgpu::VertexFormat::Uint16x2,
            AttributeData::Unorm16x2(_) => wgpu::VertexFormat::Unorm16x2,
            AttributeData::Sint16x4(_) => wgpu::VertexFormat::Sint16x4,
            AttributeData::Snorm16x4(_) => wgpu::VertexFormat::Snorm16x4,
            AttributeData::Uint16x4(_) => wgpu::VertexFormat::Uint16x4,
            AttributeData::Unorm16x4(_) => wgpu::VertexFormat::Unorm16x4,
            AttributeData::Sint8x2(_) => wgpu::VertexFormat::Sint8x2,
            AttributeData::Snorm8x2(_) => wgpu::VertexFormat::Snorm8x2,
            AttributeData::Uint8x2(_) => wgpu::VertexFormat::Uint8x2,
            AttributeData::Unorm8x2(_) => wgpu::VertexFormat::Unorm8x2,
            AttributeData::Sint8x4(_) => wgpu::VertexFormat::Sint8x4,
            AttributeData::Snorm8x4(_) => wgpu::VertexFormat::Snorm8x4,
            AttributeData::Uint8x4(_) => wgpu::VertexFormat::Uint8x4,
            AttributeData::Unorm8x4(_) => wgpu::VertexFormat::Unorm8x4,
        }
    }
}

impl AttributeData {
    pub fn byte_size(&self) -> u64 {
        let format: wgpu::VertexFormat = self.into();
        format.size()
    }
}

pub struct InterleavedAttributes {
    data: Vec<u8>,
    formats: Vec<wgpu::VertexFormat>,
    stride: u64,
}

impl InterleavedAttributes {
    pub fn from_raw_data(data: Vec<u8>, formats: &[wgpu::VertexFormat]) -> Self {
        let stride = compute_stride(formats);
        Self {
            data,
            stride,
            formats: formats.into(),
        }
    }

    pub fn with_capacity(count: u64, formats: &[wgpu::VertexFormat]) -> Self {
        let stride = compute_stride(formats);
        Self {
            data: Vec::with_capacity((count * stride) as usize),
            formats: formats.into(),
            stride,
        }
    }

    pub fn push<T: Pod>(&mut self, element: T) -> &mut Self {
        if std::mem::size_of::<T>() != self.stride as usize {
            panic!("push() called with an element that has an unexpected stide");
        }
        self.data
            .extend_from_slice(bytemuck::cast_slice(&[element]));
        self
    }

    pub fn attribute_iter<'a, T: Pod>(
        &'a self,
        attribute_id: usize,
    ) -> Result<InterleavedAttributeIter<'a, T>, ()> {
        let format = self.formats[attribute_id];
        let byte_size = format.size() as usize;
        if std::mem::size_of::<T>() != byte_size {
            return Err(());
        }

        // Compute the original byte offset.
        let mut byte_offset: usize = 0;
        for i in 0..attribute_id {
            byte_offset += format.size() as usize;
        }

        Ok(InterleavedAttributeIter {
            data: &self.data,
            byte_offset,
            byte_size,
            stride: self.stride as usize,
            _phantom_data: PhantomData,
        })
    }

    pub fn count(&self) -> usize {
        self.data.len() / self.stride as usize
    }
}

pub struct InterleavedAttributeIter<'a, T: Pod> {
    data: &'a [u8],
    byte_offset: usize,
    byte_size: usize,
    stride: usize,
    _phantom_data: PhantomData<&'a T>,
}

impl<'a, T: Pod> Iterator for InterleavedAttributeIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.byte_offset >= self.data.len() {
            return None;
        }
        let cast: &[T] =
            bytemuck::cast_slice(&self.data[self.byte_offset..self.byte_offset + self.byte_size]);

        self.byte_offset += self.stride;
        Some(&cast[0])
    }
}

pub enum Attributes {
    SoA(Vec<AttributeData>),
    Interleaved(InterleavedAttributes),
}

impl From<InterleavedAttributes> for Vec<AttributeData> {
    fn from(attr: InterleavedAttributes) -> Self {
        todo!("unimplemented")
    }
}

impl From<Vec<AttributeData>> for InterleavedAttributes {
    fn from(attr: Vec<AttributeData>) -> Self {
        todo!("unimplemented")
    }
}
