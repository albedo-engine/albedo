pub static INVALID_INDEX: u32 = std::u32::MAX;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct BVHNodeGPU {
    pub min: [f32; 3],
    pub next_node_index: u32,
    pub max: [f32; 3],
    pub primitive_index: u32,
}

impl BVHNodeGPU {
    pub fn min(&self) -> &[f32; 3] {
        &self.min
    }

    pub fn next(&self) -> u32 {
        self.next_node_index
    }

    pub fn primitive(&self) -> u32 {
        self.primitive_index
    }

    pub fn max(&self) -> &[f32; 3] {
        &self.max
    }
}

unsafe impl bytemuck::Pod for BVHNodeGPU {}
unsafe impl bytemuck::Zeroable for BVHNodeGPU {}

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
    pub fn new(world_to_model: glam::Mat4) -> Self {
        InstanceGPU {
            world_to_model,
            ..Default::default()
        }
    }
}

unsafe impl bytemuck::Pod for InstanceGPU {}
unsafe impl bytemuck::Zeroable for InstanceGPU {}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct MaterialGPU {
    pub color: glam::Vec4,
    pub roughness: f32,
    pub reflectivity: f32,
    pub albedo_texture: u32,
    pub pad_0: u32,
}
unsafe impl bytemuck::Pod for MaterialGPU {}
unsafe impl bytemuck::Zeroable for MaterialGPU {}

impl MaterialGPU {
    pub fn new(color: glam::Vec4, roughness: f32, reflectivity: f32) -> MaterialGPU {
        MaterialGPU {
            color,
            roughness,
            reflectivity,
            ..Default::default()
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct VertexGPU {
    pub position: glam::Vec3,
    padding_0: f32,
    pub normal: glam::Vec3,
    padding_1: f32,
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
            normal: (*normal).into(),
            ..Default::default()
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

unsafe impl bytemuck::Pod for LightGPU {}
unsafe impl bytemuck::Zeroable for LightGPU {}

impl LightGPU {
    pub fn new() -> Self {
        // `origin` is packed in `normal`, `tangent`, and `bitangent`.
        // By default, camera set at the origin.
        LightGPU {
            normal: glam::Vec4::new(0.0, 0.0, 1.0, 0.0),
            tangent: glam::Vec4::new(1.0, 0.0, 0.0, 0.0),
            bitangent: glam::Vec4::new(0.0, -1.0, 0.0, 0.0),
            intensity: 1.0,
            ..Default::default()
        }
    }

    pub fn from_origin(origin: glam::Vec3) -> Self {
        LightGPU {
            normal: glam::Vec4::new(0.0, 0.0, 1.0, origin.x),
            tangent: glam::Vec4::new(1.0, 0.0, 0.0, origin.y),
            bitangent: glam::Vec4::new(0.0, -1.0, 0.0, origin.z),
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
        self.bitangent = local_to_world * glam::Vec4::new(0.0, -height, 0.0, 0.0);

        origin = origin - 0.5 * self.tangent - 0.5 * self.bitangent;

        // Pack origin into the normal, tangent, and bitangent vectors.
        self.normal.w = origin.x;
        self.tangent.w = origin.y;
        self.bitangent.w = origin.z;
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct GlobalUniformsGPU {
    pub frame_count: u32,
    pub seed: u32,
    pub bounces: u32,
    pub padding: u32,
}

impl GlobalUniformsGPU {
    pub fn new() -> Self {
        GlobalUniformsGPU {
            ..Default::default()
        }
    }
}

unsafe impl bytemuck::Pod for GlobalUniformsGPU {}
unsafe impl bytemuck::Zeroable for GlobalUniformsGPU {}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct CameraGPU {
    pub origin: glam::Vec3,
    pub v_fov: f32,
    pub up: glam::Vec3,
    pub padding_0: f32,
    pub right: glam::Vec3,
    pub padding_1: f32,
    pub dimensions: [u32; 2],
    pub padding_2: [u32; 2],
}

impl Default for CameraGPU {
    fn default() -> CameraGPU {
        CameraGPU {
            origin: glam::Vec3::new(0.0, 0.0, 2.0),
            v_fov: 0.78,
            up: glam::Vec3::new(0.0, 1.0, 0.0),
            right: glam::Vec3::new(1.0, 0.0, 0.0),
            padding_0: 0.0,
            padding_1: 0.0,
            padding_2: [0, 0],
            dimensions: [1, 1],
        }
    }
}

unsafe impl bytemuck::Pod for CameraGPU {}
unsafe impl bytemuck::Zeroable for CameraGPU {}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RayGPU {
    origin: glam::Vec4,
    dir: glam::Vec4,
    radiance: glam::Vec4,
    terminated: u32,
    padding: [u32; 3],
}
unsafe impl bytemuck::Pod for RayGPU {}
unsafe impl bytemuck::Zeroable for RayGPU {}

impl RayGPU {
    pub fn new() -> Self {
        RayGPU {
            origin: glam::Vec4::new(0.0, 0.0, 0.0, 1.0),
            dir: glam::Vec4::new(0.0, 0.0, 0.0, 1.0),
            radiance: glam::Vec4::new(0.0, 0.0, 0.0, 1.0),
            terminated: 0,
            padding: [0, 0, 0],
        }
    }

    pub fn from_origin_dir(origin: &glam::Vec3, direction: glam::Vec3) -> Self {
        RayGPU {
            origin: glam::Vec4::new(origin.x, origin.y, origin.z, 1.0),
            dir: glam::Vec4::new(direction.x, direction.y, direction.z, 1.0),
            radiance: glam::Vec4::new(0.0, 0.0, 0.0, 1.0),
            terminated: 0,
            padding: [0, 0, 0],
        }
    }

    pub fn throughput(&self) -> glam::Vec3 {
        glam::Vec3::new(self.origin.w, self.dir.w, self.radiance.w)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct IntersectionGPU {
    uv: glam::Vec2,
    index: u32,
    instance: u32,
    material_index: u32,
    emitter: u32,
    dist: f32,
    padding_0: f32,
}

unsafe impl bytemuck::Pod for IntersectionGPU {}
unsafe impl bytemuck::Zeroable for IntersectionGPU {}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct TextureInfoGPU {
    x: u32,
    y: u32,
    width: u32,
    // Height should be stored in the first 24 bits, and atlas on the last 8 bits.
    layer_and_height: u32,
}

impl TextureInfoGPU {
    pub fn pack_value(layer: u32, value: u32) -> u32 {
        layer << 24 | value
    }

    pub fn unpack_value(packed: u32) -> u32 {
        packed & 0x00FFFFFF
    }

    pub fn unpack_layer(packed: u32) -> u8 {
        ((packed & 0xFF000000) >> 24) as u8
    }

    pub fn new(layer: u8, x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            layer_and_height: Self::pack_value(layer as u32, height),
        }
    }

    pub fn x(&self) -> u32 {
        self.x
    }
    pub fn y(&self) -> u32 {
        self.y
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        Self::unpack_value(self.layer_and_height)
    }
    pub fn layer(&self) -> u8 {
        Self::unpack_layer(self.layer_and_height)
    }
}

unsafe impl bytemuck::Pod for TextureInfoGPU {}
unsafe impl bytemuck::Zeroable for TextureInfoGPU {}
