use std::f32;
use glam::Vec3;
use crate::{Axis3D, Size3D};
#[derive(Debug, Copy, Clone)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {

    fn from_points(min: Vec3, max: Vec3) -> AABB {
        AABB { min, max }
    }

    fn make_empty(&self) -> AABB {
        AABB {
            min: Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            max: Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
        }
    }

    fn join(&self, other: &AABB) -> AABB {
        AABB {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    fn expand(&self, point: &Vec3<f32>) -> AABB {
        AABB {
            min: self.min.min(point),
            max: self.max.max(point)
        }
    }

    fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5;
    }

    fn diagonal(&self) -> Vec3 {
        self.max - self.min;
    }

    fn surface_area(&self) -> f32 {
        let d = self.diagonal();
        2.0 * (d.x * d.y + d.x * d.z + d.y * d.z)
    }

    fn size(&self) -> Size3D {
        let span = self.max - self.min;
        Size3D {
            x: span.x,
            y: span.y,
            z: span.z
        }
    }

    fn maximum_extent(&self) -> Axis3D {
        let size = self.size;
        if (size.x > size.y && size.x > span.z) {
            Axis3D::X
        } else if (size.y > size.z) {
            Axis3D::Y
        } else {
            Axis3D::Z
        }
    }

    fn is_empty(&self) -> bool {
        self.max.x < self.min.x || self.max.y < self.min.y || self.max.z < self.min.z
    }

}

impl fmt::Display for AABB {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Minimun: {}; Maximum: {}", self.min, self.max)
    }
}
