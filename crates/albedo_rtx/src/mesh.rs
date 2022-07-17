// @todo: how to improve separation of mesh and primitives?
// If BVH could handled sub-primitive that would be awesome.
pub trait Mesh: Sized {
    // @todo: would it be possible to allow references here in every cases?
    // What about the case where the data canno't be decayed to a &[f32; 3]?
    // @todo: make the iterator generic instead of dyn.
    fn index(&self, index: u32) -> Option<&u32>;

    // @todo: would it be possible to allow references here in every cases?
    // What about the case where the data canno't be decayed to a &[f32; 3]?
    // @todo: make the iterator generic instead of dyn.
    fn position(&self, index: u32) -> Option<&[f32; 3]>;

    // @todo: would it be possible to allow references here in every cases?
    // What about the case where the data canno't be decayed to a &[f32; 3]?
    // @todo: make the iterator generic instead of dyn.
    fn normal(&self, index: u32) -> Option<&[f32; 3]>;

    // @todo: would it be possible to allow references here in every cases?
    // What about the case where the data canno't be decayed to a &[f32; 3]?
    // @todo: make the iterator generic instead of dyn.
    fn uv(&self, index: u32) -> Option<&[f32; 2]>;

    // @todo: instead of reading vertex / buffer etc, why not ask user to fill
    // our data stucture?
    // If data are linear, user can do a memcpy, otherwise he must memcpy with
    // stride, but at least it's up to him and can give a nice perf boost.

    fn has_normal(&self) -> bool;
    fn has_tangent(&self) -> bool;
    fn has_uv0(&self) -> bool;

    fn vertex_count(&self) -> u32;
    fn index_count(&self) -> u32;
}
