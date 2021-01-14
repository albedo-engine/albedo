mod aabb;

use glam::Vec3;
use std::ops::Index;

pub use aabb::AABB;
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Size3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Axis3D {
    X = 0,
    Y = 1,
    Z = 2,
}

impl Index<Axis3D> for Vec3 {
    type Output = f32;

    fn index(&self, axis: Axis3D) -> &f32 {
        &self[axis as usize]
    }
}

pub fn clamp<T: PartialOrd>(val: T, min: T, max: T) -> T {
    if val < min {
        min
    } else if val > max {
        max
    } else {
        val
    }
}
