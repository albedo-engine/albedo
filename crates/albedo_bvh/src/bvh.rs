use albedo_math::AABB;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FlatNode {
    pub min: [f32; 3],
    pub next_node_index: u32,
    pub max: [f32; 3],
    pub primitive_index: u32,
}
unsafe impl bytemuck::Pod for FlatNode {}
unsafe impl bytemuck::Zeroable for FlatNode {}

impl FlatNode {
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

// @todo: alias std::u32::MAX with "InvalidValue" for semantic.
// @todo: make generic
pub enum Node {
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

impl Node {
    pub fn make_leaf(aabb: AABB, primitive_index: u32) -> Node {
        Node::Leaf {
            aabb,
            primitive_index,
        }
    }

    pub fn make_node(aabb: AABB) -> Node {
        Node::Node {
            aabb,
            left_child: u32::MAX,
            right_child: u32::MAX,
            forest_size: 0,
        }
    }

    pub fn aabb<'a>(&'a self) -> &'a AABB {
        match *self {
            Node::Leaf { ref aabb, .. } => &aabb,
            Node::Node { ref aabb, .. } => &aabb,
        }
    }

    pub fn primitive_index(&self) -> u32 {
        match *self {
            Node::Leaf {
                primitive_index, ..
            } => primitive_index,
            Node::Node { .. } => std::u32::MAX,
        }
    }

    pub fn forest_size(&self) -> u32 {
        match *self {
            Node::Leaf { .. } => 0,
            Node::Node { forest_size, .. } => forest_size,
        }
    }

    pub fn left_child(&self) -> Option<u32> {
        match *self {
            Node::Leaf { .. } => None,
            Node::Node { left_child, .. } => Some(left_child),
        }
    }

    pub fn right_child(&self) -> Option<u32> {
        match *self {
            Node::Leaf { .. } => None,
            Node::Node { right_child, .. } => Some(right_child),
        }
    }
}

pub struct FlatBVH {
    nodes: Vec<FlatNode>,
}

impl FlatBVH {
    pub fn nodes(&self) -> &Vec<FlatNode> {
        &self.nodes
    }
}

pub struct BVH {
    // @todo: release from CPU if not needed after build.
    pub nodes: Vec<Node>,
    root: u32,
    primitives_count: u32,
    pub(crate) flat: FlatBVH,
}

impl BVH {
    pub fn expected_nodes_count(indices_count: usize) -> usize {
        let nb_triangles = indices_count / 3;
        nb_triangles * 2 - 1
    }

    pub(crate) fn new(nodes: Vec<Node>, primitives_count: u32, root: u32) -> BVH {
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

    pub fn compute_depth(&self) -> usize {
        depth_omp(&self.nodes, self.root as usize, 0)
    }
}

fn flatten_bvh_rec(
    out: &mut Vec<FlatNode>,
    nodes: &Vec<Node>,
    input_index: u32,
    miss_index: u32,
) {
    let node = &nodes[input_index as usize];
    out.push(FlatNode {
        min: node.aabb().min.into(),
        max: node.aabb().max.into(),
        next_node_index: miss_index,
        primitive_index: node.primitive_index(),
    });

    // @todo: check that no overflow occurs
    let curr_count = out.len() as u32;

    match node {
        Node::Node {
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
                    flatten_bvh_rec(out, nodes, *left_child, miss_index);
                }
            }
            if *right_child != std::u32::MAX {
                flatten_bvh_rec(out, nodes, *right_child as u32, miss_index);
            }
        }
        _ => (),
    }
}

fn depth_omp(nodes: &[Node], input: usize, depth: usize) -> usize {
    let node = &nodes[input];
    let left_depth = if let Some(x) = node.left_child() {
        depth_omp(nodes, x as usize, depth + 1)
    } else {
        depth
    };
    let right_depth = if let Some(x) = node.right_child() {
        depth_omp(nodes, x as usize, depth + 1)
    } else {
        depth
    };
    std::cmp::max(left_depth, right_depth)
}