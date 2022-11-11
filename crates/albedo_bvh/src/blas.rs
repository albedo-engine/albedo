use crate::{FlatNode, BVH, Mesh, Vertex};
use crate::builders::{BVHBuilder};

pub struct BLASEntryDescriptor {
    pub node: u32,
    pub vertex: u32,
    pub index: u32,
}

pub struct BLASArray<Vert: Vertex> {
    pub entries: Vec<BLASEntryDescriptor>,
    pub nodes: Vec<FlatNode>,
    pub vertices: Vec<Vert>,
    pub indices: Vec<u32>,
}

impl<Vert: Vertex> BLASArray<Vert> {

    pub fn new<Builder: BVHBuilder>(meshes: &[impl Mesh<Vert>], builder: &mut Builder) -> Result<BLASArray<Vert>, &'static str> {
        let mut node_count = 0;
        let mut vertex_count = 0;
        let mut index_count = 0;

        let bvhs: Vec<BVH> = meshes
            .iter()
            .map(|mesh| -> Result<BVH, &'static str> {
                // @todo: allow user to choose builder.
                let mut bvh = builder.build(mesh)?;
                bvh.flatten();
                Ok(bvh)
            })
            .collect::<Result<Vec<BVH>, &'static str>>()?;

        let mut entries: Vec<BLASEntryDescriptor> = Vec::with_capacity(bvhs.len());
        for i in 0..bvhs.len() {
            let bvh = &bvhs[i];
            let mesh = &meshes[i];
            entries.push(BLASEntryDescriptor {
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
        let mut nodes: Vec<FlatNode> = Vec::with_capacity(node_count as usize);
        let mut vertices: Vec<Vert> = Vec::with_capacity(vertex_count as usize);
        let mut indices: Vec<u32> = Vec::with_capacity(index_count as usize);

        for i in 0..bvhs.len() {
            let bvh = &bvhs[i];
            let mesh = &meshes[i];

            nodes.extend(bvh.flat.nodes());

            // @todo: optimized: replace by memcpy when possible.
            for ii in 0..mesh.index_count() {
                indices.push(*mesh.index(ii).unwrap());
            }
            // @todo: optimized: replace by memcpy when possible.
            for v in 0..mesh.vertex_count() {
                // @todo: this assumes normal are always available.
                vertices.push(mesh.vertex(v));
            }
        }

        Ok(BLASArray {
            entries,
            nodes,
            vertices,
            indices,
        })
    }

}
