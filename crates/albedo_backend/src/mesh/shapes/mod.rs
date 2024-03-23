use super::{AttributeDescriptor, AttributeId, IndexData, Primitive, ToPrimitive};

#[derive(Debug, Copy, Clone)]
pub struct Cube {
    pub size: f32,
}

impl Cube {
    pub fn new(size: f32) -> Self {
        Self { size }
    }
}

impl ToPrimitive for Cube {
    fn to_primitive(&self, layout: &[AttributeDescriptor]) -> Result<Primitive, ()> {
        let mut primitive = Primitive::interleaved_with_count(24, layout);
        primitive.set_indices(IndexData::U16(vec![
            0, 1, 2, 2, 3, 0, // top
            4, 5, 6, 6, 7, 4, // bottom
            8, 9, 10, 10, 11, 8, // right
            12, 13, 14, 14, 15, 12, // left
            16, 17, 18, 18, 19, 16, // front
            20, 21, 22, 22, 23, 20, // back
        ]));

        match primitive.attribute_index(AttributeId::POSITION) {
            Some(index) => {
                let pos: Vec<[f32; 3]> = vec![
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
                primitive.attribute_f32x3_mut(index).copy_from_slice(&pos);
            }
            _ => return Err({}),
        };

        if let Some(index) = primitive.attribute_index(AttributeId::NORMAL) {
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
            primitive
                .attribute_f32x3_mut(index)
                .copy_from_slice(&normals);
        };

        Ok(primitive)
    }
}
