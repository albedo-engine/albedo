use crate::{Buffer, TypedBuffer};

pub enum IndexBuffer {
    U16(TypedBuffer<u16>),
    U32(TypedBuffer<u32>),
}

pub struct Primitive {
    vertices: Buffer,
    indices: IndexBuffer,
}

impl Primitive {
    pub fn new(vertex_buffer: Buffer, index_buffer: IndexBuffer) -> Self {
        Primitive {
            vertices: vertex_buffer,
            indices: index_buffer,
        }
    }

    pub fn vertices(&self) -> &crate::Buffer {
        &self.vertices
    }

    pub fn indices(&self) -> &IndexBuffer {
        &self.indices
    }
}
