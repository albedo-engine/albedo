use std::default;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct InstanceGPU {
    pub world_to_model: glam::Mat4,
    pub material_index: u32,
    // @todo: migrate those parameter to an SSBO of offsets.
    pub bvh_root_index: u32,
    pub vertex_root_index: u32,
    pub index_root_index: u32,
}

impl InstanceGPU {

    fn new(world_to_model: glam::Mat4) -> Self {
        InstanceGPU { world_to_model, ..Default::default() }
    }

}

unsafe impl bytemuck::Pod for InstanceGPU {}
unsafe impl bytemuck::Zeroable for InstanceGPU {}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MaterialGPU {
    color: glam::Vec4,
}
unsafe impl bytemuck::Pod for MaterialGPU {}
unsafe impl bytemuck::Zeroable for MaterialGPU {}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct VertexGPU {
    position: glam::Vec3,
    padding_0: u32,
    normal: glam::Vec3,
    padding_1: u32,
    // @todo: add UV
}
unsafe impl bytemuck::Pod for VertexGPU {}
unsafe impl bytemuck::Zeroable for VertexGPU {}

impl VertexGPU {

    pub fn from_position(position: &[f32; 3]) -> Self {
        VertexGPU {
            position: (*position).into(),
            ..Default::default()
        }
    }

    pub fn new(position: &[f32; 3], normal: &[f32; 3]) -> Self {
        VertexGPU {
            position: (*position).into(),
            padding_0: 0,
            normal: (*position).into(),
            padding_1: 1
        }
    }

}

impl From<&[f32; 3]> for VertexGPU {

    fn from(item: &[f32; 3]) -> Self {
        VertexGPU::from_position(item)
    }

}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct LightGPU {
    normal: glam::Vec4,
    tangent: glam::Vec4,
    bitangent: glam::Vec4,
    intensity: f32,
    padding_0: u32,
    padding_1: u32,
    padding_2: u32,
}
unsafe impl bytemuck::Pod for LightGPU {}
unsafe impl bytemuck::Zeroable for LightGPU {}
