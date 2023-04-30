use super::{AsVertexFormat, Primitive, Vertex};

fn vertex<V: Vertex>(pos: &[f32; 3], uv: &[f32; 2]) -> V {
    let mut v = V::default();
    v.set_position(pos);
    v.set_tex_coord_0(uv);
    v
}

pub trait Shape {
    fn to_interleaved_primitive<V>(&self) -> Primitive
    where
        V: Vertex + AsVertexFormat;
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
    fn to_interleaved_primitive<V>(&self) -> Primitive
    where
        V: Vertex + AsVertexFormat,
    {
        let mut attributes: Primitive =
            Primitive::interleaved_with_count(24, V::as_vertex_formats());
        // top (0, 0, 1)
        attributes.push(vertex::<V>(&[-1.0, -1.0, 1.0], &[0.0, 0.0]));
        attributes.push(vertex::<V>(&[1.0, -1.0, 1.0], &[1.0, 0.0]));
        attributes.push(vertex::<V>(&[1.0, 1.0, 1.0], &[1.0, 1.0]));
        attributes.push(vertex::<V>(&[-1.0, 1.0, 1.0], &[0.0, 1.0]));
        // bottom (0, 0, -1.0)
        attributes.push(vertex::<V>(&[-1.0, 1.0, -1.0], &[1.0, 0.0]));
        attributes.push(vertex::<V>(&[1.0, 1.0, -1.0], &[0.0, 0.0]));
        attributes.push(vertex::<V>(&[1.0, -1.0, -1.0], &[0.0, 1.0]));
        attributes.push(vertex::<V>(&[-1.0, -1.0, -1.0], &[1.0, 1.0]));
        // right (1.0, 0, 0)
        attributes.push(vertex::<V>(&[1.0, -1.0, -1.0], &[0.0, 0.0]));
        attributes.push(vertex::<V>(&[1.0, 1.0, -1.0], &[1.0, 0.0]));
        attributes.push(vertex::<V>(&[1.0, 1.0, 1.0], &[1.0, 1.0]));
        attributes.push(vertex::<V>(&[1.0, -1.0, 1.0], &[0.0, 1.0]));
        // left (-1.0, 0, 0)
        attributes.push(vertex::<V>(&[-1.0, -1.0, 1.0], &[1.0, 0.0]));
        attributes.push(vertex::<V>(&[-1.0, 1.0, 1.0], &[0.0, 0.0]));
        attributes.push(vertex::<V>(&[-1.0, 1.0, -1.0], &[0.0, 1.0]));
        attributes.push(vertex::<V>(&[-1.0, -1.0, -1.0], &[1.0, 1.0]));
        // front (0, 1.0, 0)
        attributes.push(vertex::<V>(&[1.0, 1.0, -1.0], &[1.0, 0.0]));
        attributes.push(vertex::<V>(&[-1.0, 1.0, -1.0], &[0.0, 0.0]));
        attributes.push(vertex::<V>(&[-1.0, 1.0, 1.0], &[0.0, 1.0]));
        attributes.push(vertex::<V>(&[1.0, 1.0, 1.0], &[1.0, 1.0]));
        // back (0, -1.0, 0)
        attributes.push(vertex::<V>(&[1.0, -1.0, 1.0], &[0.0, 0.0]));
        attributes.push(vertex::<V>(&[-1.0, -1.0, 1.0], &[1.0, 0.0]));
        attributes.push(vertex::<V>(&[-1.0, -1.0, -1.0], &[1.0, 1.0]));
        attributes.push(vertex::<V>(&[1.0, -1.0, -1.0], &[0.0, 1.0]));
        attributes.set_indices_u16(vec![
            0, 1, 2, 2, 3, 0, // top
            4, 5, 6, 6, 7, 4, // bottom
            8, 9, 10, 10, 11, 8, // right
            12, 13, 14, 14, 15, 12, // left
            16, 17, 18, 18, 19, 16, // front
            20, 21, 22, 22, 23, 20, // back
        ]);
        attributes
    }
}
