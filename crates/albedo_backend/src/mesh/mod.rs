use bytemuck::Pod;

pub mod shapes;

mod primitive;
pub use primitive::*;

pub trait AsVertexFormat {
    fn as_vertex_formats() -> &'static [AttributeDescriptor];
}
