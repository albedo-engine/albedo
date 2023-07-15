// #[cfg(feature = "albedo_backend")]
use albedo_backend::mesh;

//pub trait Vertex: Sized + bytemuck::Pod {}

// @todo: how to improve separation of mesh and primitives?
// If BVH could handled sub-primitive that would be awesome.
pub trait Mesh {
    // @todo: Slice of enum u32 | u16
    fn index(&self, index: u32) -> Option<u32>;
    // @todo: Return StridedSlice instead
    // @todo: would it be possible to allow references here in every cases?
    // What about the case where the data canno't be decayed to a &[f32; 3]?
    // @todo: make the iterator generic instead of dyn.
    fn position(&self, index: u32) -> Option<&[f32; 3]>;

    fn vertex_count(&self) -> u32;
    fn index_count(&self) -> u32;
}

// #[cfg(feature = "albedo_backend")]
impl Mesh for albedo_backend::mesh::Primitive {
    fn index(&self, index: u32) -> Option<u32> {
        match self.indices() {
            Some(mesh::IndexData::U16(v)) => Some(v[index as usize] as u32),
            Some(mesh::IndexData::U32(v)) => Some(v[index as usize]),
            None => None,
        }
    }

    fn position(&self, index: u32) -> Option<&[f32; 3]> {
        todo!()
        // @todo: Somehow find a way to cache position?
    }

    fn vertex_count(&self) -> u32 {
        self.vertex_count() as u32
    }

    fn index_count(&self) -> u32 {
        self.index_count() as u32
    }
}
