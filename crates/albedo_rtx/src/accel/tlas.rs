use crate::accel::BVH;
#[repr(C)]
pub struct VertexGPU {
    position: [f32; 3],
    padding_0: u32,
    normal: [f32; 3],
    padding_1: u32,
}
#[repr(C)]
pub struct BVHNodeGPU {
    min: [f32; 3],
    next_node_index: u32,
    max: [f32; 3],
    primitive_index: u32,
}

pub struct TLAS {
    _vertices: Vec<VertexGPU>,
    _indices: Vec<u32>,
}

impl TLAS {
    pub fn build(bvhs: &[BVH]) -> TLAS {}
}
