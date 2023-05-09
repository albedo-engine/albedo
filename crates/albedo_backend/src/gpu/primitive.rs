use super::{DynBuffer, IndexBuffer};

pub struct Primitive {
    pub attributes: Vec<DynBuffer>,
    pub indices: IndexBuffer,
}
