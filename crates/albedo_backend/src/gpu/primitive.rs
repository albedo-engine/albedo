use super::{BufferHandle, IndexBuffer};

pub struct Primitive {
    pub attributes: Vec<BufferHandle>,
    pub indices: IndexBuffer,
}
