use core::panic;
use std::fmt::Debug;

use bytemuck::Pod;

use crate::data::{reinterpret_vec, Slice, SliceMut};
use crate::gpu;

use super::AsVertexFormat;

fn compute_stride(formats: &[wgpu::VertexFormat]) -> usize {
    let mut stride = 0;
    for format in formats {
        stride += format.size();
    }
    stride as usize
}

fn byte_offset_for(formats: &[wgpu::VertexFormat], element: usize) -> usize {
    // Compute the original byte offset.
    let mut byte_offset = 0;
    for i in 0..element {
        byte_offset += formats[i].size();
    }
    byte_offset as usize
}

fn is_vertex_format_float(format: &wgpu::VertexFormat) -> bool {
    match format {
        wgpu::VertexFormat::Float32
        | wgpu::VertexFormat::Float64
        | wgpu::VertexFormat::Float16x2
        | wgpu::VertexFormat::Float16x4
        | wgpu::VertexFormat::Float32x2
        | wgpu::VertexFormat::Float32x3
        | wgpu::VertexFormat::Float32x4
        | wgpu::VertexFormat::Float64x2
        | wgpu::VertexFormat::Float64x3
        | wgpu::VertexFormat::Float64x4 => true,
        _ => false,
    }
}

fn is_vertex_format_unsigned(format: &wgpu::VertexFormat) -> bool {
    match format {
        wgpu::VertexFormat::Uint8x2
        | wgpu::VertexFormat::Uint8x4
        | wgpu::VertexFormat::Uint16x2
        | wgpu::VertexFormat::Uint16x4
        | wgpu::VertexFormat::Uint32
        | wgpu::VertexFormat::Uint32x2
        | wgpu::VertexFormat::Uint32x3
        | wgpu::VertexFormat::Uint32x4
        | wgpu::VertexFormat::Unorm8x2
        | wgpu::VertexFormat::Unorm8x4
        | wgpu::VertexFormat::Unorm16x2
        | wgpu::VertexFormat::Unorm16x4 => true,
        _ => false,
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
pub struct AttributeId(&'static str);

impl AttributeId {
    pub const POSITION: AttributeId = AttributeId { 0: "POSITION" };
    pub const NORMAL: AttributeId = AttributeId { 0: "NORMAL" };
    pub const TEX_COORDS_0: AttributeId = AttributeId { 0: "TEX_COORDS_0" };
}

pub struct AttributeDescriptor {
    pub id: AttributeId,
    pub format: wgpu::VertexFormat,
}

impl AttributeDescriptor {
    pub fn new<T: Into<String>>(id: AttributeId, format: wgpu::VertexFormat) -> Self {
        Self { id, format }
    }

    pub fn position(format: wgpu::VertexFormat) -> Self {
        Self {
            id: AttributeId::POSITION,
            format,
        }
    }

    pub fn normal(format: wgpu::VertexFormat) -> Self {
        Self {
            id: AttributeId::NORMAL,
            format,
        }
    }

    pub fn tex_coords_0(format: wgpu::VertexFormat) -> Self {
        Self {
            id: AttributeId::TEX_COORDS_0,
            format,
        }
    }
}

enum AttributeData {
    SoA(Vec<Vec<u8>>),
    Interleaved(Vec<u8>),
}

#[derive(Clone)]
pub enum IndexData {
    U16(Vec<u16>),
    U32(Vec<u32>),
}

#[derive(Clone)]
pub enum IndexDataSlice<'a> {
    U16(Slice<'a, u16>),
    U32(Slice<'a, u32>),
}

impl<'a> IndexDataSlice<'a> {
    pub fn len(&self) -> usize {
        match &self {
            IndexDataSlice::U16(s) => s.len(),
            IndexDataSlice::U32(s) => s.len(),
        }
    }
}

// @todo: Refactor those macros together if possible

// Macro to generate a method to lookup an attribute on a
// Primitive.
macro_rules! float_slice_attribute {
    ($name:ident, $type:ty) => {
        pub fn $name<'a>(&'a self, index: usize) -> Slice<'a, $type> {
            let format = self.attribute_format(index);
            if !is_vertex_format_float(&format) {
                panic!("attribute format isn't float"); // @todo: implement Display for VertexFormat
            }
            self.attribute::<$type>(index)
        }
    };
}
// Macro to generate a method to lookup an attribute on a
// Primitive.
macro_rules! float_slice_attribute_mut {
    ($name:ident, $type:ty) => {
        pub fn $name<'a>(&'a mut self, index: usize) -> SliceMut<'a, $type> {
            let format = self.attribute_format(index);
            if !is_vertex_format_float(&format) {
                panic!("attribute format isn't float"); // @todo: implement Display for VertexFormat
            }
            self.attribute_mut::<$type>(index)
        }
    };
}
// Macro to generate a method to lookup an attribute on a
// Primitive.
macro_rules! unsigned_slice_attribute {
    ($name:tt, $type:ty) => {
        pub fn $name<'a>(&'a mut self, index: usize) -> Slice<'a, $type> {
            let format = self.attribute_format(index);
            if !is_vertex_format_unsigned(&format) {
                panic!("attribute format isn't unsigned"); // @todo: implement Display for VertexFormat
            }
            self.attribute::<$type>(index)
        }
    };
}

pub struct Primitive {
    data: AttributeData,
    attribute_formats: Vec<wgpu::VertexFormat>,
    attribute_ids: Vec<AttributeId>,
    index_data: Option<IndexData>,
}

impl Primitive {
    fn new(data: AttributeData, descriptors: &[AttributeDescriptor]) -> Self {
        let attribute_ids: Vec<AttributeId> = descriptors.iter().map(|v| v.id).collect();
        let attribute_formats: Vec<wgpu::VertexFormat> =
            descriptors.iter().map(|v| v.format).collect();
        Self {
            data,
            attribute_formats,
            attribute_ids,
            index_data: None,
        }
    }

    pub fn interleaved<V: Pod + AsVertexFormat>(data: Vec<V>) -> Self {
        let data_u8 = reinterpret_vec(data);
        Self::new(AttributeData::Interleaved(data_u8), V::as_vertex_formats())
    }

    pub fn interleaved_with_count(count: u64, descriptors: &[AttributeDescriptor]) -> Self {
        let attribute_formats: Vec<wgpu::VertexFormat> =
            descriptors.iter().map(|v| v.format).collect();
        let byte_count = count as usize * compute_stride(&attribute_formats);
        Self::new(AttributeData::Interleaved(vec![0; byte_count]), descriptors)
    }

    pub fn soa_with_count(count: u64, descriptors: &[AttributeDescriptor]) -> Self {
        let attribute_formats: Vec<wgpu::VertexFormat> =
            descriptors.iter().map(|v| v.format).collect();
        let data: Vec<Vec<u8>> = attribute_formats
            .iter()
            .map(|v| Vec::with_capacity(v.size() as usize * count as usize))
            .collect();
        Self::new(AttributeData::SoA(data), descriptors)
    }

    pub fn attribute<'a, T: Pod>(&'a self, attribute: usize) -> Slice<'a, T> {
        let byte_size: usize = self.attribute_formats[attribute].size() as usize;
        match &self.data {
            AttributeData::Interleaved(v) => Slice::new(
                v,
                compute_stride(&self.attribute_formats),
                byte_offset_for(&self.attribute_formats, attribute),
            ),
            AttributeData::SoA(ref soa) => Slice::new(&soa[attribute], byte_size, 0),
        }
    }

    pub fn attribute_mut<'a, T: Pod>(&'a mut self, attribute: usize) -> SliceMut<'a, T> {
        let byte_size: usize = self.attribute_formats[attribute].size() as usize;
        match &mut self.data {
            AttributeData::Interleaved(v) => SliceMut::new(
                v,
                compute_stride(&self.attribute_formats),
                byte_offset_for(&self.attribute_formats, attribute),
            ),
            AttributeData::SoA(ref mut soa) => SliceMut::new(&mut soa[attribute], byte_size, 0),
        }
    }

    float_slice_attribute!(attribute_f32, f32);
    float_slice_attribute!(attribute_f32x2, [f32; 2]);
    float_slice_attribute!(attribute_f32x3, [f32; 3]);
    float_slice_attribute!(attribute_f32x4, [f32; 4]);
    float_slice_attribute!(attribute_f64, f64);
    float_slice_attribute!(attribute_f64x2, [f64; 2]);
    float_slice_attribute!(attribute_f64x3, [f64; 3]);
    float_slice_attribute!(attribute_f64x4, [f64; 4]);

    float_slice_attribute_mut!(attribute_f32_mut, f32);
    float_slice_attribute_mut!(attribute_f32x2_mut, [f32; 2]);
    float_slice_attribute_mut!(attribute_f32x3_mut, [f32; 3]);
    float_slice_attribute_mut!(attribute_f32x4_mut, [f32; 4]);
    float_slice_attribute_mut!(attribute_f64_mut, f64);
    float_slice_attribute_mut!(attribute_f64x2_mut, [f64; 2]);
    float_slice_attribute_mut!(attribute_f64x3_mut, [f64; 3]);
    float_slice_attribute_mut!(attribute_f64x4_mut, [f64; 4]);

    unsigned_slice_attribute!(attribute_u32, u32);
    unsigned_slice_attribute!(attribute_u32x2, [u32; 2]);
    unsigned_slice_attribute!(attribute_u32x3, [u32; 3]);
    unsigned_slice_attribute!(attribute_u32x4, [u32; 4]);
    // @todo: implement missing attributes

    // @todo: add overload to move a Vec ownership into
    // the primtive when using SOA.

    pub fn attribute_index(&self, id: AttributeId) -> Option<usize> {
        self.attribute_ids.iter().position(|&val| val == id)
    }

    pub fn attribute_format(&self, index: usize) -> wgpu::VertexFormat {
        self.attribute_formats[index]
    }

    pub fn attribute_count(&self) -> usize {
        self.attribute_formats.len()
    }

    pub fn set_indices(&mut self, data: IndexData) {
        self.index_data = Some(data);
    }

    pub fn set_indices_u16(&mut self, data: Vec<u16>) {
        self.index_data = Some(IndexData::U16(data));
    }

    pub fn set_indices_u32(&mut self, data: Vec<u32>) {
        self.index_data = Some(IndexData::U32(data));
    }

    pub fn indices(&self) -> Option<&IndexData> {
        self.index_data.as_ref()
    }

    pub fn vertex_count(&self) -> usize {
        match &self.data {
            AttributeData::Interleaved(ref v) => v.len() / compute_stride(&self.attribute_formats),
            AttributeData::SoA(ref v) => v[0].len() / self.attribute_formats[0].size() as usize,
        }
    }

    pub fn index_count(&self) -> usize {
        match &self.index_data {
            Some(IndexData::U16(v)) => v.len(),
            Some(IndexData::U32(v)) => v.len(),
            None => 0,
        }
    }

    pub fn is_interleaved(&self) -> bool {
        match self.data {
            AttributeData::Interleaved(_) => true,
            _ => false,
        }
    }
}

impl gpu::AsVertexBufferLayout for Primitive {
    fn as_vertex_buffer_layout(&self) -> gpu::VertexBufferLayoutBuilder {
        let mut builder = gpu::VertexBufferLayoutBuilder::new(self.attribute_count());
        for i in 0..self.attribute_count() {
            builder = builder.auto_attribute(self.attribute_format(i));
        }
        builder
    }
}

pub struct PrimitiveResourceBuilder<'a> {
    primitive: &'a Primitive,
    descriptor: Option<gpu::BufferInitDescriptor<'a>>,
}

impl<'a> PrimitiveResourceBuilder<'a> {
    pub fn new(primitive: &'a Primitive) -> Self {
        Self {
            primitive,
            descriptor: None,
        }
    }

    pub fn descriptor(mut self, desc: gpu::BufferInitDescriptor<'a>) -> Self {
        self.descriptor = Some(desc);
        self
    }
}

impl<'a> gpu::ResourceBuilder for PrimitiveResourceBuilder<'a> {
    type Resource = gpu::Primitive;

    fn build(self, device: &wgpu::Device) -> Result<Self::Resource, String> {
        let mut attributes = vec![];
        let descriptor = if let Some(desc) = self.descriptor {
            desc
        } else {
            gpu::BufferInitDescriptor::new(Some("Primitive Buffer"), wgpu::BufferUsages::VERTEX)
        };

        let count: usize = self.primitive.vertex_count();

        attributes.push(match &self.primitive.data {
            AttributeData::Interleaved(v) => {
                gpu::DynBuffer::new_with_data(device, v, count as u64, Some(descriptor))
            }
            AttributeData::SoA(ref _soa) => {
                todo!("unimplemented")
            }
        });

        // @todo: no unwrap.
        let indices = match self.primitive.index_data.as_ref().unwrap() {
            IndexData::U16(v) => gpu::IndexBuffer::new_with_data_16(device, &v, Some(descriptor)),
            IndexData::U32(v) => gpu::IndexBuffer::new_with_data_32(device, &v, Some(descriptor)),
        };

        Ok(gpu::Primitive {
            attributes,
            indices,
        })
    }
}

impl Debug for IndexData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::U16(arg0) => f.debug_list().entries(arg0).finish(),
            Self::U32(arg0) => f.debug_list().entries(arg0).finish(),
        }
    }
}
