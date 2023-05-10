use albedo_rtx::uniforms;
use libc;
use std::{os::raw, sync::Mutex};

mod baker;
pub use baker::*;

mod app;
pub use app::*;

#[repr(C)]
pub struct Slice<'a, T> {
    count: usize,
    data: &'a mut [T],
}

impl<'a, T> Slice<'a, T> {
    pub fn data(&self) -> &[T] {
        &self.data
    }
    pub fn data_mut(&mut self) -> &mut [T] {
        &mut self.data
    }
}

#[repr(C)]
pub struct ImageSlice<'a> {
    width: u32,
    height: u32,
    data: &'a mut [f32],
}

impl<'a> ImageSlice<'a> {
    pub fn data(&self) -> &[f32] {
        &self.data
    }
    pub fn data_mut(&mut self) -> &mut [f32] {
        &mut self.data
    }
}

#[no_mangle]
#[repr(C)]
pub struct MeshDescriptor {
    positions: *const f32,
    normals: *const f32,
    indices: *const u32,
    vertex_count: u32,
    index_count: u32,
}

static app: Mutex<Option<App>> = Mutex::new(None);

#[no_mangle]
pub extern "C" fn init() {
    println!("Hello from Rust");
    unsafe {
        *app.lock().unwrap() = Some(App::new());
    }
}

pub extern "C" fn set_mesh_data(desc: MeshDescriptor) {
    let count = desc.vertex_count / 3;
    if count % 3 != 0 {
        panic!("Vertex count must be a multiple of 3");
    }

    println!("Seting mesh data...");

    let raw_indices =
        unsafe { std::slice::from_raw_parts(desc.indices, desc.index_count as usize) };
    let raw_positions =
        unsafe { std::slice::from_raw_parts(desc.positions, desc.vertex_count as usize) };
    let raw_normals =
        unsafe { std::slice::from_raw_parts(desc.normals, desc.vertex_count as usize) };

    // @todo: Skip conversion by making the BVH / GPU struct split the vertex.
    let mut vertices: Vec<uniforms::Vertex> = Vec::with_capacity(count as usize);
    for j in 0..count {
        let i = j as usize * 3;
        let pos = [raw_positions[i], raw_positions[i + 1], raw_positions[i + 2]];
        let normal = [raw_normals[i], raw_normals[i + 1], raw_normals[i + 2]];
        vertices.push(uniforms::Vertex::new(&pos, &normal, None));
    }

    let mut guard = app.lock().unwrap();
    let runtime = guard.as_mut().unwrap();
    let baker = runtime.baker_mut();
    baker.set_mesh_data(runtime.device(), &vertices, raw_indices);
}

pub extern "C" fn bake(raw_slice: *mut ImageSlice) {
    println!("Baking...");
    if raw_slice.is_null() {
        return;
    }
    println!("Baking...2");
    let mut guard = app.lock().unwrap();
    let runtime = guard.as_mut().unwrap();

    let slice = unsafe { raw_slice.as_mut() }.unwrap();
    runtime.baker().bake_into(runtime.device(), slice);
}
