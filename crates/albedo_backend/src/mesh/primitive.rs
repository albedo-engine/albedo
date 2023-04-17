use std::marker::PhantomData;

use bytemuck::Pod;

use crate::data::InterleavedVec;

pub trait AsVertexFormats {
    fn as_vertex_formats() -> &'static [wgpu::VertexFormat];
}

struct AttributeDescriptor {
    name: String,
    format: wgpu::VertexFormat,
}

pub fn convert_descriptors(
    descriptors: &[AttributeDescriptor],
) -> (Vec<String>, Vec<wgpu::VertexFormat>) {
    let attribute_names = descriptors.iter().map(|v| v.name).collect();
    let attribute_formats = descriptors.iter().map(|v| v.format).collect();
    (attribute_names, attribute_formats)
}

fn compute_stride(formats: &[wgpu::VertexFormat]) -> usize {
    let mut stride = 0;
    for format in formats {
        stride += format.size();
    }
    stride as usize
}

enum AttributeData {
    SoA(Vec<Vec<u8>>),
    Interleaved(InterleavedVec),
    Chunk(Vec<u8>),
}

enum IndexData {
    U16(Vec<u16>),
    U32(Vec<u32>),
}

pub struct Primitive {
    data: AttributeData,
    attribute_formats: Vec<wgpu::VertexFormat>,
    attribute_names: Vec<String>,
    index_data: Option<IndexData>,
}

impl Primitive {
    pub fn interleaved_with_count(count: u64, descriptors: &[AttributeDescriptor]) -> Self {
        let (attribute_names, attribute_formats) = convert_descriptors(descriptors);
        let sizes: Vec<usize> = attribute_formats
            .iter()
            .map(|v| v.size() as usize)
            .collect();
        Self {
            data: AttributeData::Interleaved(InterleavedVec::with_capacity(count as usize, sizes)),
            attribute_formats,
            attribute_names, // stride,
            index_data: None,
        }
    }

    pub fn push<T: Pod>(&mut self, element: T) -> &mut Self {
        match &self.data {
            AttributeData::Interleaved(data) => data.push(element),
            _ => panic!(),
        };
        self
    }

    pub fn get_value<V: Pod>(&self, attribute_id: usize, index: usize) -> &V {
        todo!("unimplemented")
    }

    pub fn get_value_mut<V: Pod>(&mut self, attribute_id: usize, index: usize) -> &mut V {
        todo!("unimplemented")
    }

    pub fn attribute_iter<'a, T: Pod>(
        &'a self,
        attribute_id: usize,
    ) -> Result<PrimitiveIter<'a, &'a T>, ()> {
        let byte_size = self.attribute_formats[attribute_id].size() as usize;
        if std::mem::size_of::<T>() != byte_size {
            return Err(());
        }
        Ok(match &self.data {
            AttributeData::Interleaved(v) => PrimitiveIter {
                data: v.data(),
                stride: v.stride(),
                byte_offset: v.byte_offset_for(attribute_id),
                byte_end: v.data().len(),
                _phantom_data: PhantomData,
            },
            AttributeData::Chunk(v) => {
                todo!("unimplemented")
            }
            AttributeData::SoA(soa) => {
                let v = &soa[attribute_id];
                PrimitiveIter {
                    data: v.data(),
                    stride: byte_size,
                    byte_offset: 0,
                    byte_end: v.data().len(),
                    _phantom_data: PhantomData,
                }
            }
        })
    }
}

pub struct PrimitiveIter<'a, T> {
    data: &'a [u8],
    byte_offset: usize,
    byte_end: usize,
    stride: usize,
    _phantom_data: PhantomData<&'a T>,
}

impl<'a, T> Iterator for PrimitiveIter<'a, T>
where
    T: Pod,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.byte_offset >= self.byte_end {
            return None;
        }
        let cast: &[T] =
            bytemuck::cast_slice(&self.data[self.byte_offset..self.byte_offset + self.byte_size]);

        self.byte_offset += self.stride;
        Some(cast[0])
    }
}
