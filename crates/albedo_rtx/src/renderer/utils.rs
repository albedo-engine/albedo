use crate::accel::BVH;
use crate::mesh::Mesh;

struct Offsets {
    vertex: u32,
    index: u32,
}

pub fn build_acceleration_structure_gpu<'a>(bvhs: &'a [(&'a BVH, &'a impl Mesh<'a>)]) {
    let mut vertex_count = 0;
    let mut index_count = 0;

    let mut offsets: Vec<Offsets> = Vec::with_capacity(bvhs.len());
    for tuple in bvhs {
        offsets.push(Offsets {
            vertex: vertex_count,
            index: index_count,
        });
        index_count += tuple.0.primitives_count();
        vertex_count += tuple.1.vertex_count();
    }

    // let vertices = Vec::new
}
