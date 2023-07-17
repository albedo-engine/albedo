use super::{AttributeDescriptor, AttributeId, IndexData, Primitive};

#[derive(Debug, Clone)]
pub struct ShapeData {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: IndexData,
}

impl ShapeData {
    pub fn count(&self) -> u64 {
        self.positions.len() as u64
    }

    pub fn to_primitive(self, layout: &[AttributeDescriptor]) -> Result<Primitive, ()> {
        let mut primitive = Primitive::interleaved_with_count(self.count(), layout);
        primitive.set_indices(self.indices);

        match primitive.attribute_index(AttributeId::POSITION) {
            Some(index) => {
                primitive.attribute_f32x3_mut(index).set(&self.positions);
            }
            _ => return Err({}),
        };

        if let Some(index) = primitive.attribute_index(AttributeId::NORMAL) {
            primitive.attribute_f32x3_mut(index).set(&self.normals);
        };

        Ok(primitive)
    }
}

pub trait Shape {
    fn data(&self) -> ShapeData;
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
    fn data(&self) -> ShapeData {
        let positions: Vec<[f32; 3]> = vec![
            // top (0, 0, 1)
            [-1.0, -1.0, 1.0],
            [1.0, -1.0, 1.0],
            [1.0, 1.0, 1.0],
            [-1.0, 1.0, 1.0],
            // bottom (0, 0, -1.0)
            [-1.0, 1.0, -1.0],
            [1.0, 1.0, -1.0],
            [1.0, -1.0, -1.0],
            [-1.0, -1.0, -1.0],
            // right (1.0, 0, 0)
            [1.0, -1.0, -1.0],
            [1.0, 1.0, -1.0],
            [1.0, 1.0, 1.0],
            [1.0, -1.0, 1.0],
            // left (-1.0, 0, 0)
            [-1.0, -1.0, 1.0],
            [-1.0, 1.0, 1.0],
            [-1.0, 1.0, -1.0],
            [-1.0, -1.0, -1.0],
            // front (0, 1.0, 0)
            [1.0, 1.0, -1.0],
            [-1.0, 1.0, -1.0],
            [-1.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
            // back (0, -1.0, 0)
            [1.0, -1.0, 1.0],
            [-1.0, -1.0, 1.0],
            [-1.0, -1.0, -1.0],
            [1.0, -1.0, -1.0],
        ];

        let normals: Vec<[f32; 3]> = vec![
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
        ];

        let indices = IndexData::U16(vec![
            0, 1, 2, 2, 3, 0, // top
            4, 5, 6, 6, 7, 4, // bottom
            8, 9, 10, 10, 11, 8, // right
            12, 13, 14, 14, 15, 12, // left
            16, 17, 18, 18, 19, 16, // front
            20, 21, 22, 22, 23, 20, // back
        ]);
        ShapeData {
            positions,
            normals,
            indices,
        }
    }
}
