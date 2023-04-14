#[derive(Debug, Copy, Clone)]
pub struct Box {
    pub min: [f32; 3],
    pub max: [f32; 3],
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

impl From<Cube> for Box {
    fn from(primitive: Cube) -> Self {
        let pos_half = primitive.size * 0.5;
        let neg_half = -pos_half;
        Box {
            min: [neg_half, neg_half, neg_half],
            max: [pos_half, pos_half, pos_half],
        }
    }
}
