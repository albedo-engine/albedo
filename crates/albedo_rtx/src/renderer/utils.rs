use crate::accel::BVH;
use crate::mesh::Mesh;
use crate::renderer::resources;

pub struct Offsets {
    node: u32,
    vertex: u32,
    index: u32,
}

impl Offsets {
    pub fn node(&self) -> u32 {
        self.node
    }

    pub fn vertex(&self) -> u32 {
        self.vertex
    }

    pub fn index(&self) -> u32 {
        self.index
    }
}

pub struct GPUResources {
    pub offset_table: Vec<Offsets>,
    pub nodes_buffer: Vec<resources::BVHNodeGPU>,
    pub vertex_buffer: Vec<resources::VertexGPU>,
    pub index_buffer: Vec<u32>,
}

// @todo: move to bvh crate.
// @todo: passing BVH and meshes separately makes it possible to feed a BVH that
// doesn't go with the associated mesh...
// However, the nature of the BVH makes it disociated from its mesh.
pub fn build_acceleration_structure_gpu<'a>(bvhs: &[BVH], meshes: &[impl Mesh]) -> GPUResources {
    let mut node_count = 0;
    let mut vertex_count = 0;
    let mut index_count = 0;

    assert!(meshes.len() >= bvhs.len());

    let mut offset_table: Vec<Offsets> = Vec::with_capacity(bvhs.len());
    for i in 0..bvhs.len() {
        let bvh = &bvhs[i];
        let mesh = &meshes[i];
        offset_table.push(Offsets {
            node: node_count,
            vertex: vertex_count,
            index: index_count,
        });
        // @todo: check for u32 overflow.
        node_count += bvh.nodes.len() as u32;
        index_count += mesh.index_count();
        vertex_count += mesh.vertex_count();
    }

    // @todo: parallel for.
    let mut nodes_buffer: Vec<resources::BVHNodeGPU> = Vec::with_capacity(node_count as usize);
    let mut vertex_buffer: Vec<resources::VertexGPU> = Vec::with_capacity(vertex_count as usize);
    let mut index_buffer: Vec<u32> = Vec::with_capacity(index_count as usize);

    for i in 0..bvhs.len() {
        let bvh = &bvhs[i];
        let mesh = &meshes[i];

        nodes_buffer.extend(bvh.flat.nodes());

        // @todo: optimized: replace by memcpy when possible.
        for ii in 0..mesh.index_count() {
            index_buffer.push(*mesh.index(ii).unwrap());
        }
        // @todo: optimized: replace by memcpy when possible.
        for v in 0..mesh.vertex_count() {
            // @todo: this assumes normal are always available.
            vertex_buffer.push(resources::VertexGPU::new(
                mesh.position(v).unwrap(),
                mesh.normal(v).unwrap(),
                mesh.uv(v)
            ));
        }
    }

    GPUResources {
        offset_table,
        nodes_buffer,
        vertex_buffer,
        index_buffer,
    }
}
