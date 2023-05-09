#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex {
    pub pos: [f32; 4],
    pub uv: [f32; 2],
}

impl Vertex {
    fn new(position: [f32; 3], uv: [f32; 2]) -> Vertex {
        Vertex {
            pos: [position[0], position[1], position[2], 1.0],
            uv,
        }
    }
}

unsafe impl bytemuck::Zeroable for Vertex {}
unsafe impl bytemuck::Pod for Vertex {}

pub trait Geometry {
    fn vertices<'a>(&'a self) -> &'a [Vertex];
    fn indices<'a>(&'a self) -> &'a [u16];
}

pub struct CubeGeometry {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

impl CubeGeometry {
    pub fn indices() -> Vec<u16> {
        [
            0, 1, 2, 2, 3, 0, // top
            4, 5, 6, 6, 7, 4, // bottom
            8, 9, 10, 10, 11, 8, // right
            12, 13, 14, 14, 15, 12, // left
            16, 17, 18, 18, 19, 16, // front
            20, 21, 22, 22, 23, 20, // back
        ]
        .to_vec()
    }

    pub fn vertices() -> Vec<Vertex> {
        [
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
        ]
        .to_vec()
    }

    pub fn new() -> CubeGeometry {
        CubeGeometry {
            vertices: Self::vertices(),
            indices: Self::indices(),
        }
    }
}

impl Geometry for CubeGeometry {
    fn vertices<'a>(&'a self) -> &[Vertex] {
        self.vertices.as_slice()
    }
    fn indices<'a>(&'a self) -> &[u16] {
        self.indices.as_slice()
    }
}
