mod sah_bvh_builder;

pub use sah_bvh_builder::SAHBuilder;
pub use crate::{BVH, Mesh, Vertex};

pub trait BVHBuilder {
    // @todo: create custom Error type.
    fn build<V: Vertex>(&mut self, mesh: &impl Mesh<V>) -> Result<BVH, &'static str>;
}
