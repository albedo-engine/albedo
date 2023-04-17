use super::{Primitive, Vertex};

fn vertex<V: Vertex>(pos: &[f32; 3]) -> V {
    let mut v = V::default();
    v.set_position(pos);
    v.set_normal(pos);
    v
}

pub trait Shape {
    fn to_interleaved_primitive<V>() -> Primitive;
}

#[derive(Debug, Copy, Clone)]
pub struct Cube {
    pub size: f32,
}

impl Cube {
    pub fn new(size: f32) -> Self {
        Self { size }
    }
}

impl Shape for Cube {
    fn to_interleaved_primitive<V: Vertex>() -> Primitive {
        let mut attributes = Primitive::interleaved_with_count(24, V::as_vertex_formats());
        // top (0, 0, 1)
        Vertex::new([-1.0, -1.0, 1.0], [0.0, 0.0]),
        Vertex::new([1.0, -1.0, 1.0], [1.0, 0.0]),
        Vertex::new([1.0, 1.0, 1.0], [1.0, 1.0]),
        Vertex::new([-1.0, 1.0, 1.0], [0.0, 1.0]),
        // bottom (0, 0, -1.0)
        Vertex::new([-1.0, 1.0, -1.0], [1.0, 0.0]),
        Vertex::new([1.0, 1.0, -1.0], [0.0, 0.0]),
        Vertex::new([1.0, -1.0, -1.0], [0.0, 1.0]),
        Vertex::new([-1.0, -1.0, -1.0], [1.0, 1.0]),
        // right (1.0, 0, 0)
        Vertex::new([1.0, -1.0, -1.0], [0.0, 0.0]),
        Vertex::new([1.0, 1.0, -1.0], [1.0, 0.0]),
        Vertex::new([1.0, 1.0, 1.0], [1.0, 1.0]),
        Vertex::new([1.0, -1.0, 1.0], [0.0, 1.0]),
        // left (-1.0, 0, 0)
        Vertex::new([-1.0, -1.0, 1.0], [1.0, 0.0]),
        Vertex::new([-1.0, 1.0, 1.0], [0.0, 0.0]),
        Vertex::new([-1.0, 1.0, -1.0], [0.0, 1.0]),
        Vertex::new([-1.0, -1.0, -1.0], [1.0, 1.0]),
        // front (0, 1.0, 0)
        Vertex::new([1.0, 1.0, -1.0], [1.0, 0.0]),
        Vertex::new([-1.0, 1.0, -1.0], [0.0, 0.0]),
        Vertex::new([-1.0, 1.0, 1.0], [0.0, 1.0]),
        Vertex::new([1.0, 1.0, 1.0], [1.0, 1.0]),
        // back (0, -1.0, 0)
        Vertex::new([1.0, -1.0, 1.0], [0.0, 0.0]),
        Vertex::new([-1.0, -1.0, 1.0], [1.0, 0.0]),
        Vertex::new([-1.0, -1.0, -1.0], [1.0, 1.0]),
        Vertex::new([1.0, -1.0, -1.0], [0.0, 1.0]),
    }
}
