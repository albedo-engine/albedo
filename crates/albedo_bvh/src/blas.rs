use crate::builders::BVHBuilder;
use crate::{BVHNode, Mesh};

/// Node, vertex, and index offset of an entry
///
/// This is used to retrieve a flattened BVH into a buffer
pub struct BLASEntryDescriptor {
    pub node: u32,
    pub vertex: u32,
    pub index: u32,
}

/// Data-oriented storage for a list of BVH.
///
/// Data are stored in separate buffers:
///
/// `[vertex_0, vertex_1, vertex_2, ..., vertex_n]`
/// `[index_0, index_1, index_2, ..., index_j]`
/// `[entry_0, entry_1, entry_2, ..., entry_k]`
///
/// Entries are used to find the start index of each
/// BVH.
pub struct BLASArray<Vert: bytemuck::Pod> {
    /// Node, vertex, and index offset for each entry
    pub entries: Vec<BLASEntryDescriptor>,
    /// List of nodes of all entries
    pub nodes: Vec<BVHNode>,
    /// List of vertices of all entries
    pub vertices: Vec<Vert>,
    /// List of indices of all entries
    pub indices: Vec<u32>,
}

impl<Vert> BLASArray<Vert>
where
    Vert: bytemuck::Pod,
{
    pub fn new<Builder: BVHBuilder>(
        meshes: &[impl Mesh<Vert>],
        builder: &mut Builder,
    ) -> Result<BLASArray<Vert>, &'static str> {
        let mut node_count = 0;
        let mut vertex_count = 0;
        let mut index_count = 0;

        let bvhs: Vec<Vec<BVHNode>> = meshes
            .iter()
            .map(|mesh| -> Result<Vec<BVHNode>, &'static str> {
                // @todo: allow user to choose builder.
                let mut bvh = builder.build(mesh)?;
                Ok(bvh.flatten())
            })
            .collect::<Result<Vec<Vec<BVHNode>>, &'static str>>()?;

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
            node_count += bvh.len() as u32;
            index_count += mesh.index_count();
            vertex_count += mesh.vertex_count();
        }

        // @todo: parallel for.
        let mut nodes: Vec<BVHNode> = Vec::with_capacity(node_count as usize);
        let mut vertices: Vec<Vert> = Vec::with_capacity(vertex_count as usize);
        let mut indices: Vec<u32> = Vec::with_capacity(index_count as usize);

        for i in 0..bvhs.len() {
            let mesh = &meshes[i];

            nodes.extend(&bvhs[i]);

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
