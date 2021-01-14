pub struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
}

// @todo: how to improve separation of mesh and primitives?
// If BVH could handled sub-primitive that would be awesome.
pub trait Mesh: Sized {
    // @todo: would it be possible to allow references here in every cases?
    // What about the case where the data canno't be decayed to a &[f32; 3]?
    // @todo: make the iterator generic instead of dyn.
    fn iter_indices(&self) -> dyn Iterator<Item = u32>;

    // @todo: would it be possible to allow references here in every cases?
    // What about the case where the data canno't be decayed to a &[f32; 3]?
    // @todo: make the iterator generic instead of dyn.
    fn iter_positions(&self) -> dyn Iterator<Item = &[f32; 3]>;

    // @todo: would it be possible to allow references here in every cases?
    // What about the case where the data canno't be decayed to a &[f32; 3]?
    // @todo: make the iterator generic instead of dyn.
    //
    // @todo: directly send a GPU vertex to avoid copy.
    fn iter_vertex(&self) -> dyn Iterator<Item = Vertex>;
}
