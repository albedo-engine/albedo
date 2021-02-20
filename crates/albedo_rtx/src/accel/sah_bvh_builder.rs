use albedo_math::{clamp, AABB};
use glam::Vec3;

use crate::accel::{BVHBuilder, BVHNode, BVH};
use crate::mesh::Mesh;

#[derive(Default, Copy, Clone)]
struct SAHBin {
    aabb: AABB,
    primitives_count: u32,
    right_cost: f32,
}
// @todo: allow to change bin size with const generics?
pub struct SAHBuilder {
    _bins: [SAHBin; 8],
}

impl SAHBuilder {
    pub fn new() -> SAHBuilder {
        SAHBuilder {
            _bins: [SAHBin::default(); 8],
        }
    }
}

impl BVHBuilder for SAHBuilder {
    fn build<'a, T: Mesh<'a>>(&mut self, mesh: &'a T) -> Result<BVH, &'static str> {
        // @todo: support for quads.
        // @todo: support for u8 and u32.
        let nb_triangles = mesh.iter_indices_u32().count() / 3;
        if nb_triangles == 0 {
            return Err("todo");
        }

        let nodes_count = 2 * nb_triangles - 1;
        let mut nodes = Vec::with_capacity(nodes_count);

        // @todo: this assumes model is triangulated. Fix that.
        // Creates all leaf nodes.
        let mut indices = mesh.iter_indices_u32();
        for i in 0..nb_triangles {
            let i0 = (*indices.next().unwrap()) as usize;
            let i1 = (*indices.next().unwrap()) as usize;
            let i2 = (*indices.next().unwrap()) as usize;
            let v0_pos = mesh.position(i0).unwrap();
            let v1_pos = mesh.position(i1).unwrap();
            let v2_pos = mesh.position(i2).unwrap();

            let mut aabb = AABB::make_empty();
            aabb.expand_mut(&Vec3::from(*v0_pos));
            aabb.expand_mut(&Vec3::from(*v1_pos));
            aabb.expand_mut(&Vec3::from(*v2_pos));
            nodes.push(BVHNode::make_leaf(aabb, i as u32));
        }

        let root = rec_build(&mut nodes, &mut self._bins, 0, nodes_count);

        Ok(BVH::new(nodes, nb_triangles, root))
    }
}

fn rec_build(nodes: &mut Vec<BVHNode>, bins: &mut [SAHBin], start: usize, end: usize) -> usize {
    if end - start <= 1 {
        return start;
    }

    let mut aabb = AABB::make_empty();
    let mut centroids = AABB::make_empty();
    for i in start..end {
        let node = &nodes[i];
        aabb.join_mut(node.aabb());
        // @todo: cache center computation.
        centroids.expand_mut(&node.aabb().center());
    }

    let curr_node_index = nodes.len();
    nodes.push(BVHNode::make_node(aabb));

    // The split is based on the largest dimension.
    let split_axis = centroids.maximum_extent();
    let split_axis_len = centroids.max[split_axis] - centroids.min[split_axis];

    //
    // Step 1: initializes every bin computing, for each triangle, its associated
    // bin. Each bin bounding box and number of primitives is updated.
    //

    // @todo: figure out why automatic re-borrowing doesn't work here?
    for bin in &mut *bins {
        bin.primitives_count = 0;
        bin.aabb = AABB::make_empty();
    }

    for i in start..end {
        let node = &nodes[i];
        // @todo: cache center computation.
        let center_on_axis = node.aabb().center()[split_axis];
        let bin_index = get_bin_index(center_on_axis, centroids.min[split_axis], split_axis_len);
        let bin = &mut bins[bin_index];
        bin.primitives_count += 1;
        bin.aabb.join_mut(node.aabb());
    }

    let split_index = find_best_split(bins);
    let mut middle = partition(&mut nodes[start..=end], |val| {
        let center_on_axis = val.aabb().center()[split_axis];
        // @todo: cache center computation
        let i = get_bin_index(center_on_axis, centroids.min[split_axis], split_axis_len);
        i < split_index
    });
    if middle <= start || middle >= end {
        middle = (start + end) / 2;
    }

    let left_child_index = rec_build(nodes, bins, start, middle);
    let right_child_index = rec_build(nodes, bins, middle, end);
    let left_surface_area = nodes[left_child_index].aabb().surface_area();
    let left_forest_size = nodes[left_child_index].forest_size();
    let right_surface_area = nodes[right_child_index].aabb().surface_area();
    let right_forest_size = nodes[right_child_index].forest_size();

    if let BVHNode::Node {
        ref mut forest_size,
        ref mut left_child,
        ref mut right_child,
        ..
    } = nodes[curr_node_index]
    {
        *forest_size += 1 + left_forest_size + 1 + right_forest_size;
        *left_child = left_child_index as u32;
        *right_child = right_child_index as u32;
        if right_surface_area > left_surface_area {
            *left_child = right_child_index as u32;
            *right_child = left_child_index as u32;
        }
    }

    0
}

fn find_best_split(bins: &mut [SAHBin]) -> usize {
    // @todo: use const generics to take bin count into account.
    const BIN_COUNT: usize = 8;

    let mut aabb = AABB::make_empty();
    let mut primitives_count = 0;

    //
    // Step 1: save the cost of splitting starting from the right side.
    // the cost is directly stored into the bin. By doing so, we can save one
    // stack allocation.
    //

    for i in (1..=(BIN_COUNT - 1)).rev() {
        let bin = &mut bins[i];
        aabb.join_mut(&bin.aabb);
        primitives_count += bin.primitives_count;
        bin.right_cost = (primitives_count as f32) * aabb.surface_area();
    }

    //
    // Step 2: compute the cost of splitting from the left side.
    //
    // Compute the overall left + right side cost at each bin, and save the
    // lowest overall cost.
    //

    primitives_count = 0;
    aabb = AABB::make_empty();

    let mut split_index = 0;
    let mut min_cost = f32::INFINITY;

    for i in 0..(BIN_COUNT - 1) {
        let bin = &bins[i];
        aabb.join_mut(&bin.aabb);
        primitives_count += bin.primitives_count;
        // SAH theory states that the cost is relative to the probability of
        // intersecting the sub area. However, we are simply comparing the cost,
        // so the division can be skipped.
        let cost = ((primitives_count as f32) * aabb.surface_area()) + bins[i + 1].right_cost;
        if cost < min_cost {
            min_cost = cost;
            split_index = i + 1;
        }
    }

    split_index
}

fn get_bin_index(
    split_axis_aabb_center: f32,
    split_axis_centroids_min: f32,
    split_axis_len: f32,
) -> usize {
    let normalized = (split_axis_aabb_center - split_axis_centroids_min) / split_axis_len;
    // @todo: use const generics to take bin count into account.
    clamp((normalized * 7.0) as usize, 0, 7)
}

fn partition<T, P>(arr: &mut [T], p: P) -> usize
where
    P: Fn(&T) -> bool,
{
    let mut last_index = 0;
    for i in 0..arr.len() {
        if p(&arr[i]) {
            arr.swap(i, last_index);
            last_index += 1;
        }
    }
    last_index
}
