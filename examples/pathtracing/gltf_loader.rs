use albedo_rtx::mesh::Mesh;
use gltf;
use std::path::Path;

pub struct ProxyMesh {
    positions: Vec<[f32; 3]>,
    indices: Vec<u32>,
}
impl<'a> Mesh<'a> for ProxyMesh {
    type IndexIter = std::slice::Iter<'a, u32>;

    fn iter_indices_u32(&'a self) -> Self::IndexIter {
        self.indices.iter()
    }

    fn position(&self, index: usize) -> Option<&[f32; 3]> {
        self.positions.get(index)
    }
}

pub struct Scene {
    pub meshes: Vec<ProxyMesh>,
}

pub fn load_gltf<P: AsRef<Path>>(file_path: &P) -> Scene {
    let (doc, buffers, images) = match gltf::import(file_path) {
        Ok(tuple) => tuple,
        Err(err) => {
            panic!("glTF import failed: {:?}", err);
            // if let gltf::Error::Io(_) = err {
            //     error!("Hint: Are the .bin file(s) referenced by the .gltf file available?")
            // }
        }
    };
    let mut meshes: Vec<ProxyMesh> = Vec::new();
    for mesh in doc.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            meshes.push(ProxyMesh {
                positions: reader.read_positions().unwrap().collect(),
                indices: reader
                    .read_indices()
                    .map(|read_indices| read_indices.into_u32().collect::<Vec<_>>())
                    .unwrap(),
            });
        }
    }
    Scene { meshes }
}
