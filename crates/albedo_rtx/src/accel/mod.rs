use albedo_math::AABB;
use crate::Mesh;

// @todo: make generic
enum BVHNode {
    Leaf {
        primitive_index: u32,
    },
    Node {
        bounds: AABB,
        left_child: u32,
        right_child: u32,
        forest_size: u32,
    },
}

impl BVHNode {

    fn new() -> BVHNode {
        BVHNode {
            left_child: std::u32::MAX,
            right_child: std::u32::MAX,
            forest_size: std::u32::MAX,
        }
    }

}
pub struct BVH {
    pub nodes: Vec<BVHNode>,
}

pub trait BVHBuilder {

    fn build(&mesh: Mesh) -> BVH;

}
