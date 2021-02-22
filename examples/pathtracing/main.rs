use albedo_rtx::accel::{BVHBuilder, SAHBuilder, BVH};

mod gltf_loader;
use gltf_loader::load_gltf;

fn main() {
    let scene = load_gltf(&"./examples/pathtracing/assets/box.glb");

    let bvhs: Vec<BVH> = scene
        .meshes
        .iter()
        .map(|mesh| {
            let mut builder = SAHBuilder::new();
            builder.build(mesh).unwrap()
        })
        .collect();

    for n in &bvhs[0].nodes {
        println!("{}", n.aabb());
    }
}
