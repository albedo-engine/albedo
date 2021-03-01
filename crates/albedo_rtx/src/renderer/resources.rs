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
#[derive(Clone, Copy, Default)]
pub struct LightGPU {
    pub normal: glam::Vec4,
    pub tangent: glam::Vec4,
    pub bitangent: glam::Vec4,
    pub intensity: f32,
    padding_0: u32,
    padding_1: u32,
    padding_2: u32,
}

impl LightGPU {

    pub fn new() -> Self {
        // `origin` is packed in `normal`, `tangent`, and `bitangent`.
        // By default, camera set at the origin.
        LightGPU {
            normal: glam::Vec4::new(0.0, 0.0, 1.0, 0.0),
            tangent: glam::Vec4::new(1.0,0.0, 0.0, 0.0),
            bitangent: glam::Vec4::new(0.0, - 1.0, 0.0, 0.0),
            intensity: 1.0,
            ..Default::default()
        }
    }

    pub fn from_origin(origin: glam::Vec3) -> Self {
        LightGPU {
            normal: glam::Vec4::new(0.0, 0.0, 1.0, origin.x),
            tangent: glam::Vec4::new(1.0,0.0, 0.0, origin.y),
            bitangent: glam::Vec4::new(0.0, - 1.0, 0.0, origin.z),
            intensity: 1.0,
            ..Default::default()
        }
    }

    pub fn from_matrix(local_to_world: glam::Mat4) -> Self {
        let mut light = LightGPU::new();
        light.set_from_matrix(local_to_world, 1.0, 1.0);
        light
    }

    pub fn set_from_matrix(&mut self, local_to_world: glam::Mat4, width: f32, height: f32) {
        let mut origin = local_to_world.w_axis;
        self.normal = local_to_world * glam::Vec4::new(0.0, 0.0, 1.0, 0.0);
        self.tangent = local_to_world * glam::Vec4::new(width, 0.0, 0.0, 0.0);
        self.bitangent = local_to_world * glam::Vec4::new(0.0, - height, 0.0, 0.0);

        origin = origin - 0.5 * self.tangent - 0.5 * self.bitangent;

        // Pack origin into the normal, tangent, and bitangent vectors.
        self.normal.w = origin.x;
        self.tangent.w = origin.y;
        self.bitangent.w = origin.z;
    }

}

unsafe impl bytemuck::Pod for LightGPU {}
unsafe impl bytemuck::Zeroable for LightGPU {}
