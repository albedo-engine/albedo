// @todo: how to improve separation of mesh and primitives?
// If BVH could handled sub-primitive that would be awesome.
pub trait Mesh<'a>: Sized {
    type IndexIter: Iterator<Item = &'a u32>;

    // @todo: would it be possible to allow references here in every cases?
    // What about the case where the data canno't be decayed to a &[f32; 3]?
    // @todo: make the iterator generic instead of dyn.
    fn iter_indices_u32(&'a self) -> Self::IndexIter;

    // @todo: would it be possible to allow references here in every cases?
    // What about the case where the data canno't be decayed to a &[f32; 3]?
    // @todo: make the iterator generic instead of dyn.
    fn position(&'a self, index: usize) -> Option<&[f32; 3]>;

    // @todo: would it be possible to allow references here in every cases?
    // What about the case where the data canno't be decayed to a &[f32; 3]?
    // @todo: make the iterator generic instead of dyn.
    //
    // @todo: directly send a GPU vertex to avoid copy.
    // fn iter_vertex(&self) -> dyn Iterator<Item = Vertex>;
}
