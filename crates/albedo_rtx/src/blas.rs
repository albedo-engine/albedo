use tinybvh_rs::{PrimitiveCWBVH, CWBVH};

use crate::{uniforms::Instance, BVHNode, Vertex};

pub struct MeshDescriptor<'a> {
    pub positions: pas::Slice<'a, [f32; 4]>,
    pub normals: Option<pas::Slice<'a, [f32; 3]>>,
    pub texcoords0: Option<pas::Slice<'a, [f32; 2]>>,
}

pub struct IndexedMeshDescriptor<'a> {
    pub mesh: MeshDescriptor<'a>,
    pub indices: &'a [u32],
}

/// Node, vertex, and index offset of an entry
///
/// This is used to retrieve a flattened BVH into a buffer
#[derive(Default)]
pub struct BLASEntryDescriptor {
    pub node: u32,
    pub primitive: u32,
    pub vertex: u32,
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
#[derive(Default)]
pub struct BLASArray {
    /// Node, vertex, and index offset for each entry
    pub entries: Vec<BLASEntryDescriptor>,
    /// List of nodes of all entries
    pub nodes: Vec<BVHNode>,
    /// List of indices of all entries
    pub primitives: Vec<PrimitiveCWBVH>,
    pub vertices: Vec<Vertex>,
    pub instances: Vec<Instance>,
}

impl BLASArray {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn add_bvh(&mut self, mesh: MeshDescriptor) {
        self.entries.push(BLASEntryDescriptor {
            node: self.nodes.len() as u32,
            primitive: self.primitives.len() as u32,
            vertex: self.vertices.len() as u32,
        });

        let start = self.vertices.len();
        self.vertices
            .resize(start + mesh.positions.len() / 3, Vertex::default());
        let vertices: &mut [Vertex] = &mut self.vertices[start..];

        for i in 0..mesh.positions.len() {
            let pos = &mesh.positions[i];
            vertices[i].position = [pos[0], pos[1], pos[2], 0.0];
        }
        if let Some(normals) = mesh.normals {
            for i in 0..normals.len() {
                let normal = &normals[i];
                vertices[i].normal = [normal[0], normal[1], normal[2], 0.0];
            }
        }
        if let Some(texcoord) = mesh.texcoords0 {
            for i in 0..texcoord.len() {
                let uv = &texcoord[i];
                vertices[i].position[3] = uv[0];
                vertices[i].normal[3] = uv[1];
            }
        }
        let bvh = CWBVH::new_strided(&mesh.positions);
        self.add_bvh_internal(bvh);
    }

    pub fn add_bvh_indexed(&mut self, desc: IndexedMeshDescriptor) {
        self.entries.push(BLASEntryDescriptor {
            node: self.nodes.len() as u32,
            primitive: self.primitives.len() as u32,
            vertex: self.vertices.len() as u32,
        });

        let vertex_count = desc.indices.len();
        let start = self.vertices.len();
        self.vertices
            .resize(start + vertex_count, Vertex::default());

        let vertices: &mut [Vertex] = &mut self.vertices[start..];
        for (i, index) in desc.indices.into_iter().enumerate() {
            let position = &desc.mesh.positions[*index as usize];
            vertices[i].position = *position;
        }
        if let Some(normals) = desc.mesh.normals {
            for (i, index) in desc.indices.into_iter().enumerate() {
                let normal = &normals[*index as usize];
                vertices[i].normal = [normal[0], normal[1], normal[2], 0.0];
            }
        }
        if let Some(uvs) = desc.mesh.texcoords0 {
            for (i, index) in desc.indices.into_iter().enumerate() {
                let uv = &uvs[*index as usize];
                vertices[i].position[3] = uv[0];
                vertices[i].normal[3] = uv[1];
            }
        }

        let vertices: &[Vertex] = &self.vertices[start..];
        let positions: pas::Slice<[f32; 4]> = pas::Slice::new(vertices, 0);

        let bvh = CWBVH::new_strided(&positions);
        self.add_bvh_internal(bvh);
    }

    pub fn add_instance(&mut self, bvh_index: usize, model_to_world: glam::Mat4, material: u32) {
        let entry = self.entries.get(bvh_index).unwrap();
        self.instances.push(Instance {
            model_to_world,
            world_to_model: model_to_world.inverse(),
            material_index: material,
            bvh_root_index: entry.node,
            vertex_root_index: entry.vertex,
            bvh_primitive_index: entry.primitive,
        });
    }

    fn add_bvh_internal(&mut self, bvh: CWBVH) {
        self.nodes.extend(bvh.nodes());
        self.primitives.extend(bvh.primitives());
    }
}
