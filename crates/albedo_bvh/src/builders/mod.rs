mod sah_bvh_builder;

pub use crate::{Mesh, BVH};
pub use sah_bvh_builder::SAHBuilder;

pub trait BVHBuilder {
    // @todo: create custom Error type.
    fn build<V: bytemuck::Pod>(&mut self, mesh: &impl Mesh<V>) -> Result<BVH, &'static str>;
}
