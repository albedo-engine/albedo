use bytemuck::Pod;

pub mod shapes;

mod primitive;
pub use primitive::*;

pub trait AsVertexFormat {
    fn as_vertex_formats() -> &'static [AttributeDescriptor];
}

pub trait Vertex: Pod + Default {
    fn set_position(&mut self, pos: &[f32; 3]);
    fn set_normal(&mut self, pos: &[f32; 3]) {}
    fn set_tex_coord_0(&mut self, pos: &[f32; 2]) {}
}
