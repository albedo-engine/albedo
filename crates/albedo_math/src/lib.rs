mod aabb;

pub use aabb::{AABB};
pub struct Size3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
pub enum Axis3D {
    X,
    Y,
    Z
}

