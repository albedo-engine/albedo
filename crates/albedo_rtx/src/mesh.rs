pub struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
}

// @todo: how to improve separation of mesh and primitives?
// If BVH could handled sub-primitive that would be awesome.
pub trait Mesh: Sized {

    type IndexIter: Iterator<Item = u16>;
    type PositionIter: Iterator<Item = [f32; 3]>;

    // @todo: would it be possible to allow references here in every cases?
    // What about the case where the data canno't be decayed to a &[f32; 3]?
    // @todo: make the iterator generic instead of dyn.
    fn iter_indices_u16(&self) -> Self::IndexIter;

    // @todo: would it be possible to allow references here in every cases?
    // What about the case where the data canno't be decayed to a &[f32; 3]?
    // @todo: make the iterator generic instead of dyn.
    fn iter_positions(&self) -> Self::PositionIter;

    // @todo: would it be possible to allow references here in every cases?
    // What about the case where the data canno't be decayed to a &[f32; 3]?
    // @todo: make the iterator generic instead of dyn.
    //
    // @todo: directly send a GPU vertex to avoid copy.
    // fn iter_vertex(&self) -> dyn Iterator<Item = Vertex>;
}
