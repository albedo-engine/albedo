pub mod macros;
pub mod passes;
pub mod texture;
pub mod uniforms;

pub use uniforms::*;

use albedo_backend::gpu;

pub fn get_dispatch_size(
    size: (u32, u32, u32),
    workgroup_size: (u32, u32, u32),
) -> (u32, u32, u32) {
    let x: f32 = (size.0 as f32) / workgroup_size.0 as f32;
    let y: f32 = (size.1 as f32) / workgroup_size.1 as f32;
    let z: f32 = (size.2 as f32) / workgroup_size.2 as f32;
    return (x.ceil() as u32, y.ceil() as u32, z.ceil() as u32);
}

pub struct SceneLayout {
    layout: wgpu::BindGroupLayout,
}

pub struct SceneBindGroup<'a> {
    layout: &'a wgpu::BindGroupLayout,
    bindings: Vec<wgpu::BindGroupEntry<'a>>,
}

impl SceneLayout {
    const PER_DRAW_STRUCT_BINDING: u32 = 0;

    const INSTANCE_BINDING: u32 = 0;
    const NODE_BINDING: u32 = 1;
    const INDEX_BINDING: u32 = 2;
    const VERTEX_BINDING: u32 = 3;

    const LIGHT_BINDING: u32 = 0;
    const MATERIAL_BINDING: u32 = 1;
    const TEXTURE_PROBE_BINDING: u32 = 2;
    const TEXTURE_INFO_BINDING: u32 = 3;
    const TEXTURE_ATLAS_BINDING: u32 = 4;
    const SAMPLER_BINDING: u32 = 5;
    const SAMPLER_LINEAR_BINDING: u32 = 6;

    pub fn buffer(binding: u32, visibility: wgpu::ShaderStages, read_only: bool) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    pub fn texture(binding: u32, visibility: wgpu::ShaderStages, sample_type: wgpu::TextureSampleType, view_dimension: wgpu::TextureViewDimension) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                sample_type,
                view_dimension,
            },
            count: None,
        }
    }

    pub fn sampler(binding: u32, visibility: wgpu::ShaderStages, binding_type: wgpu::SamplerBindingType) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Sampler(binding_type),
            count: None,
        }
    }

    pub fn create(device: &wgpu::Device) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Scene Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: Self::PER_DRAW_STRUCT_BINDING,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                Self::buffer(Self::INSTANCE_BINDING, wgpu::ShaderStages::COMPUTE,
                    true),
                Self::buffer(Self::NODE_BINDING, wgpu::ShaderStages::COMPUTE,
                    true),
                Self::buffer(Self::INDEX_BINDING, wgpu::ShaderStages::COMPUTE,
                    true),
                Self::buffer(Self::VERTEX_BINDING, wgpu::ShaderStages::COMPUTE,
                    true),
                Self::buffer(Self::LIGHT_BINDING, wgpu::ShaderStages::COMPUTE,
                    true),
                Self::buffer(Self::MATERIAL_BINDING, wgpu::ShaderStages::COMPUTE,
                    true),
                Self::texture(Self::TEXTURE_PROBE_BINDING, wgpu::ShaderStages::COMPUTE,
                    wgpu::TextureSampleType::Float { filterable: true }, wgpu::TextureViewDimension::D2),
                Self::texture(Self::TEXTURE_INFO_BINDING, wgpu::ShaderStages::COMPUTE,
                    wgpu::TextureSampleType::Uint, wgpu::TextureViewDimension::D1),
                Self::texture(Self::TEXTURE_ATLAS_BINDING, wgpu::ShaderStages::COMPUTE,
                    wgpu::TextureSampleType::Float { filterable: true }, wgpu::TextureViewDimension::D2Array),
                Self::sampler(Self::SAMPLER_BINDING, wgpu::ShaderStages::COMPUTE,
                    wgpu::SamplerBindingType::Filtering),
                Self::sampler(Self::SAMPLER_LINEAR_BINDING, wgpu::ShaderStages::COMPUTE,
                    wgpu::SamplerBindingType::Filtering),
            ],
        });
        Self { layout }
    }
    
    pub fn bind_group(&self, bindings_count: u32) -> SceneBindGroup {
        SceneBindGroup{
            layout: &self.layout,
            bindings: Vec::<wgpu::BindGroupEntry>::with_capacity(bindings_count as usize)
        }
    }
}

impl<'a> SceneBindGroup<'a> {
    pub fn create(
        &self,
        device: &wgpu::Device,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Scene Bind Group"),
            layout: &self.layout,
            entries: &self.bindings.as_slice(),
        })
    }

    pub fn buffer(binding: u32, buffer: &'a wgpu::Buffer) -> wgpu::BindGroupEntry<'a> {
        wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: &buffer,
                offset: 0,
                size: None,
            }),
        }
    }

    pub fn texture(binding: u32, texture_view: &'a wgpu::TextureView) -> wgpu::BindGroupEntry<'a> {
        wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::TextureView(texture_view),
        }
    }

    pub fn sampler(binding: u32, sampler: &'a wgpu::Sampler) -> wgpu::BindGroupEntry<'a> {
        wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::Sampler(sampler),
        }
    }

    pub fn instances(mut self, instances: &'a gpu::Buffer<uniforms::Instance>) -> Self {
        self.bindings.push(Self::buffer(SceneLayout::INSTANCE_BINDING, instances.inner()));
        self
    }

    pub fn nodes(mut self, nodes: &'a wgpu::Buffer) -> Self {
        self.bindings.push(Self::buffer(SceneLayout::NODE_BINDING, nodes));
        self
    }

    pub fn indices(mut self, indices: &'a gpu::Buffer<u32>) -> Self {
        self.bindings.push(Self::buffer(SceneLayout::INDEX_BINDING, indices.inner()));
        self
    }

    pub fn vertices(mut self, vertices: &'a wgpu::Buffer) -> Self {
        self.bindings.push(Self::buffer(SceneLayout::VERTEX_BINDING, vertices));
        self
    }

    pub fn lights(mut self, lights: &'a gpu::Buffer<uniforms::Light>) -> Self {
        self.bindings.push(Self::buffer(SceneLayout::LIGHT_BINDING, lights.inner()));
        self
    }

    pub fn materials(mut self, materials: &'a gpu::Buffer<uniforms::Material>) -> Self {
        self.bindings.push(Self::buffer(SceneLayout::MATERIAL_BINDING, materials.inner()));
        self
    }

    pub fn textures(
        mut self,
        probe_view: &'a wgpu::TextureView,
        texture_info: &'a wgpu::TextureView,
        atlas_view: &'a wgpu::TextureView,
        sampler_nearest: &'a wgpu::Sampler,
        sampler_linear: &'a wgpu::Sampler,
    ) -> Self {
        self.bindings.extend_from_slice(&[
                Self::texture(SceneLayout::TEXTURE_PROBE_BINDING, probe_view),
                Self::texture(SceneLayout::TEXTURE_INFO_BINDING, texture_info),
                Self::texture(SceneLayout::TEXTURE_ATLAS_BINDING, atlas_view),
                Self::sampler(SceneLayout::SAMPLER_BINDING, sampler_nearest),
                Self::sampler(SceneLayout::SAMPLER_LINEAR_BINDING, sampler_linear),
            ]);
        self
    }

    pub fn uniforms(
        mut self,
        global_uniforms: &'a gpu::Buffer<uniforms::PerDrawUniforms>,
    ) -> Self {
        self.bindings.push(
                wgpu::BindGroupEntry {
                    binding: SceneLayout::PER_DRAW_STRUCT_BINDING,
                    resource: global_uniforms.as_entire_binding(),
                });
        self
    }
}
