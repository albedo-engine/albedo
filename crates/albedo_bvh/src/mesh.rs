//pub trait Vertex: Sized + bytemuck::Pod {}

// @todo: how to improve separation of mesh and primitives?
// If BVH could handled sub-primitive that would be awesome.
pub trait Mesh<V : bytemuck::Pod> {
    // @todo: would it be possible to allow references here in every cases?
    // What about the case where the data canno't be decayed to a &[f32; 3]?
    // @todo: make the iterator generic instead of dyn.
    fn index(&self, index: u32) -> Option<&u32>;
    fn position(&self, index: u32) -> Option<&[f32; 3]>;
    fn vertex(&self, index: u32) -> V;

    fn vertex_count(&self) -> u32;
    fn index_count(&self) -> u32;
}
