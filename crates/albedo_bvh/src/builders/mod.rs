mod sah_bvh_builder;

pub use sah_bvh_builder::SAHBuilder;
pub use crate::{BVH, Mesh};


trait Builder {

    fn build(mesh: &impl Mesh) -> BVH; 

}
