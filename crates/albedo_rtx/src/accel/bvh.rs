use std::cmp::max;

use crate::mesh::Mesh;
use albedo_math::AABB;

// @todo: alias std::u32::MAX with "InvalidValue" for semantic.
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
            primitive_index,
        }
    }

    pub fn make_node(aabb: AABB) -> BVHNode {
        BVHNode::Node {
            aabb,
            left_child: u32::MAX,
            right_child: u32::MAX,
            forest_size: 0,
        }
    }

    pub fn aabb<'a>(&'a self) -> &'a AABB {
        match *self {
            BVHNode::Leaf { ref aabb, .. } => &aabb,
            BVHNode::Node { ref aabb, .. } => &aabb,
        }
    }

    pub fn primitive_index(&self) -> u32 {
        match *self {
            BVHNode::Leaf {
                primitive_index, ..
            } => primitive_index,
            BVHNode::Node { .. } => std::u32::MAX,
        }
    }

    pub fn forest_size(&self) -> u32 {
        match *self {
            BVHNode::Leaf { .. } => 0,
            BVHNode::Node { forest_size, .. } => forest_size,
        }
    }

    pub fn left_child(&self) -> Option<u32> {
        match *self {
            BVHNode::Leaf { .. } => None,
            BVHNode::Node { left_child, .. } => Some(left_child),
        }
    }

    pub fn right_child(&self) -> Option<u32> {
        match *self {
            BVHNode::Leaf { .. } => None,
            BVHNode::Node { right_child, .. } => Some(right_child),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BVHNodeGPU {
    min: [f32; 3],
    next_node_index: u32,
    max: [f32; 3],
    primitive_index: u32,
}

impl BVHNodeGPU {
    pub fn min(&self) -> &[f32; 3] {
        &self.min
    }

    pub fn next(&self) -> u32 {
        self.next_node_index
    }

    pub fn primitive(&self) -> u32 {
        self.primitive_index
    }

    pub fn max(&self) -> &[f32; 3] {
        &self.max
    }
}

unsafe impl bytemuck::Pod for BVHNodeGPU {}
unsafe impl bytemuck::Zeroable for BVHNodeGPU {}

pub struct FlatBVH {
    nodes: Vec<BVHNodeGPU>,
}

impl FlatBVH {
    pub fn nodes(&self) -> &Vec<BVHNodeGPU> {
        &self.nodes
    }
}

pub struct BVH {
    // @todo: release from CPU if not needed after build.
    pub nodes: Vec<BVHNode>,
    root: u32,
    primitives_count: u32,
    pub(crate) flat: FlatBVH,
}

impl BVH {
    pub fn expected_nodes_count(indices_count: usize) -> usize {
        let nb_triangles = indices_count / 3;
        nb_triangles * 2 - 1
    }

    pub(crate) fn new(nodes: Vec<BVHNode>, primitives_count: u32, root: u32) -> BVH {
        let count = nodes.len();
        BVH {
            nodes,
            primitives_count,
            root,
            flat: FlatBVH {
                nodes: Vec::with_capacity(count),
            },
        }
    }

    pub fn flatten(&mut self) {
        self.flat.nodes.clear();
        self.flat.nodes.reserve_exact(self.nodes.len());

        println!("Depth = {}", depth_omp(&self.nodes, self.root() as usize, 0));

        flatten_bvh_rec(
            &mut self.flat.nodes,
            &self.nodes,
            self.root as u32,
            std::u32::MAX,
        );
    }

    pub fn primitives_count(&self) -> u32 {
        self.primitives_count
    }

    pub fn root(&self) -> u32 {
        self.root
    }
}

pub trait BVHBuilder {
    // @todo: create custom Error type.
    fn build(&mut self, mesh: &impl Mesh) -> Result<BVH, &'static str>;
}

fn flatten_bvh_rec(
    out: &mut Vec<BVHNodeGPU>,
    nodes: &Vec<BVHNode>,
    inputIndex: u32,
    missIndex: u32,
) {
    let node = &nodes[inputIndex as usize];
    out.push(BVHNodeGPU {
        min: node.aabb().min.into(),
        max: node.aabb().max.into(),
        next_node_index: missIndex,
        primitive_index: node.primitive_index(),
    });

    // @todo: check that no overflow occurs
    let curr_count = out.len() as u32;

    match node {
        BVHNode::Node {
            left_child,
            right_child,
            ..
        } => {
            if *left_child != std::u32::MAX {
                let left_node = &nodes[*left_child as usize];
                if *right_child != std::u32::MAX {
                    let miss_idx = left_node.forest_size() + curr_count + 1;
                    flatten_bvh_rec(out, nodes, *left_child, miss_idx);
                } else {
                    flatten_bvh_rec(out, nodes, *left_child, missIndex);
                }
            }
            if *right_child != std::u32::MAX {
                flatten_bvh_rec(out, nodes, *right_child as u32, missIndex);
            }
        }
        _ => (),
    }
}

fn depth_omp(nodes: &[BVHNode], input: usize, depth: usize) -> usize {
    let node = &nodes[input];
    let left_depth = if let Some(x) = node.left_child() {
        depth_omp(nodes, x as usize, depth + 1)
    } else {
        0 as usize
    };
    let right_depth = if let Some(x) = node.right_child() { depth_omp(nodes, x as usize, depth + 1) } else { 0 as usize };
    depth + std::cmp::max(left_depth, right_depth)
}
