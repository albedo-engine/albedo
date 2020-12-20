use glam::Vec3;
use albedo_math::AABB;

use crate::Mesh;
use crate::accel::{BVH, BVHNode, BVHBuilder};
pub struct SAHBuilder {}

impl<T: Mesh> BVHBuilder<T> for SAHBuilder {

    fn build(mesh: &T) -> Result<BVH, &'static str> {
        let indices = mesh.get_indices();
        let nb_triangles = indices.len() / 3;
        if nb_triangles == 0 {
            return Ok(BVH {
                nodes: Vec::with_capacity(0)
            });
        }

        let nodes_count = 2 * nb_triangles - 1;
        let mut nodes = Vec::with_capacity(nodes_count);

        // @todo: this assumes model is triangulated. Fix that.
        // Creates all leaf nodes.
        for i in (0..indices.len()).step_by(3) {
            let v0_pos = mesh.get_position(indices[i]).unwrap();
            let v1_pos = mesh.get_position(indices[i + 1]).unwrap();
            let v2_pos = mesh.get_position(indices[i + 2]).unwrap();

            let mut aabb = AABB::make_empty();
            aabb.expand_mut(&Vec3::from(*v0_pos));
            aabb.expand_mut(&Vec3::from(*v1_pos));
            aabb.expand_mut(&Vec3::from(*v2_pos));
            nodes.push(BVHNode::make_leaf(aabb, i as u32));
        }
        Ok(BVH {
            nodes
        })
    }

}


