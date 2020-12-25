use std::f32;
use std::fmt;
use glam::Vec3;
use crate::{Axis3D, Size3D};
#[derive(Debug, Copy, Clone)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {

    pub fn from_points(min: Vec3, max: Vec3) -> AABB {
        AABB { min, max }
    }

    pub fn make_empty() -> AABB {
        AABB {
            min: Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            max: Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
        }
    }

    pub fn join(&self, other: &AABB) -> AABB {
        AABB {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    pub fn join_mut(&mut self, other: &AABB) {
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
    }

    pub fn expand(&self, point: &Vec3) -> AABB {
        AABB {
            min: self.min.min(*point),
            max: self.max.max(*point)
        }
    }

    pub fn expand_mut(&mut self, point: &Vec3) {
        self.min = self.min.min(*point);
        self.max = self.max.max(*point);
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn diagonal(&self) -> Vec3 {
        self.max - self.min
    }

    pub fn surface_area(&self) -> f32 {
        let d = self.diagonal();
        2.0 * (d.x * d.y + d.x * d.z + d.y * d.z)
    }

    pub fn size(&self) -> Size3D {
        let span = self.max - self.min;
        Size3D {
            x: span.x,
            y: span.y,
            z: span.z
        }
    }

    pub fn maximum_extent(&self) -> Axis3D {
        let size = self.size();
        if size.x > size.y && size.x > size.z {
            Axis3D::X
        } else if size.y > size.z {
            Axis3D::Y
        } else {
            Axis3D::Z
        }
    }

    pub fn is_empty(&self) -> bool {
        self.max.x < self.min.x || self.max.y < self.min.y || self.max.z < self.min.z
    }

}

impl Default for AABB {

    fn default() -> Self {
        AABB::make_empty()
    }

}

impl fmt::Display for AABB {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Minimun: {}; Maximum: {}", self.min, self.max)
    }
}
