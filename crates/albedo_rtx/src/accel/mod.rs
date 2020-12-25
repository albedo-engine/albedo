mod sah_bvh_builder;

use glam::Vec3;
use albedo_math::AABB;
use crate::Mesh;

pub use sah_bvh_builder::SAHBuilder;

// @todo: make generic
pub enum BVHNode {
    Leaf {
        aabb: AABB,
        primitive_index: u32,
    },
    Node {
        aabb: AABB,
        left_child: u32,
        right_child: u32,
        forest_size: u32,
    },
}

impl BVHNode {

    pub fn make_leaf(aabb: AABB, primitive_index: u32) -> BVHNode {
        BVHNode::Leaf {
            aabb,
            primitive_index
        }
    }

    pub fn make_node(aabb: AABB) -> BVHNode {
        BVHNode::Node {
            aabb,
            left_child: u32::MAX,
            right_child: u32::MAX,
            forest_size: 0
        }
    }

    pub fn aabb<'a>(&'a self) -> &'a AABB {
        match *self {
            BVHNode::Leaf{ ref aabb, .. } => &aabb,
            BVHNode::Node{ ref aabb, .. } => &aabb,
        }
    }

    pub fn forest_size(&self) -> u32 {
        match *self {
            BVHNode::Leaf{..} => 0,
            BVHNode::Node{ forest_size, .. } => forest_size,
        }
    }

}

pub struct BVH {
    pub nodes: Vec<BVHNode>,
}

pub trait BVHBuilder<T: Mesh> {

    // @todo: create custom Error type.
    fn build(&mut self, mesh: &T) -> Result<BVH, &'static str>;

}
