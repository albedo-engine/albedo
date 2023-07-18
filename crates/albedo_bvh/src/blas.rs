use albedo_backend::mesh::IndexDataSlice;

use crate::builders::BVHBuilder;
use crate::{BVHNode, Mesh};

/// Node, vertex, and index offset of an entry
///
/// This is used to retrieve a flattened BVH into a buffer
#[derive(Default)]
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
pub struct BLASArray {
    /// Node, vertex, and index offset for each entry
    pub entries: Vec<BLASEntryDescriptor>,
    /// List of nodes of all entries
    pub nodes: Vec<BVHNode>,
    /// List of indices of all entries
    pub indices: Vec<u32>,
    vertex_count: usize,
}

impl BLASArray {
    pub fn empty() -> Self {
        Self {
            entries: vec![BLASEntryDescriptor {
                node: crate::INVALID_INDEX,
                vertex: crate::INVALID_INDEX,
                index: crate::INVALID_INDEX,
            }],
            nodes: vec![BVHNode {
                ..Default::default()
            }],
            indices: vec![crate::INVALID_INDEX],
            vertex_count: Default::default(),
        }
    }

    pub fn new<Builder: BVHBuilder>(
        meshes: &[impl Mesh],
        builder: &mut Builder,
    ) -> Result<BLASArray, &'static str> {
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
            index_count += mesh.indices().unwrap().len() as u32;
            vertex_count += mesh.positions().unwrap().len() as u32;
        }

        // @todo: parallel for.
        let mut nodes: Vec<BVHNode> = Vec::with_capacity(node_count as usize);
        let mut indices: Vec<u32> = Vec::with_capacity(index_count as usize);

        for i in 0..bvhs.len() {
            let mesh = &meshes[i];
            nodes.extend(&bvhs[i]);
            // @todo: optimized: replace by memcpy when possible.
            let slice = mesh.indices().unwrap();
            match slice {
                IndexDataSlice::U16(v) => {
                    for i in v.iter() {
                        indices.push(*i as u32);
                    }
                }
                IndexDataSlice::U32(v) => {
                    for i in v.iter() {
                        indices.push(*i);
                    }
                }
            };
        }

        Ok(BLASArray {
            entries,
            nodes,
            indices,
            vertex_count: vertex_count as usize,
        })
    }

    pub fn vertex_count(&self) -> usize {
        self.vertex_count
    }
}
