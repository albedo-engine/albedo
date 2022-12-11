use crate::macros::path_separator;
use crate::uniforms::{PerDrawUniforms, Ray};
use albedo_backend::{GPUBuffer, UniformBuffer};

pub struct AccumulationPass {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
}

impl AccumulationPass {
    const RAY_BINDING: u32 = 0;
    const PER_DRAW_STRUCT_BINDING: u32 = 1;
    const TEXTURE_BINDING: u32 = 2;
    const READ_TEXTURE_BINDING: u32 = 3;
    const SAMPLER_BINDING: u32 = 4;

    pub fn new(device: &wgpu::Device, source: Option<wgpu::ShaderModuleDescriptor>) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Accumulation Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: Self::RAY_BINDING,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: Self::TEXTURE_BINDING,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        format: wgpu::TextureFormat::Rgba32Float,
                        #[cfg(target_arch = "wasm32")]
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        #[cfg(not(target_arch = "wasm32"))]
                        access: wgpu::StorageTextureAccess::ReadWrite,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
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
                #[cfg(target_arch = "wasm32")]
                wgpu::BindGroupLayoutEntry {
                    binding: Self::READ_TEXTURE_BINDING,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                #[cfg(target_arch = "wasm32")]
                wgpu::BindGroupLayoutEntry {
                    binding: Self::SAMPLER_BINDING,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Accumulation Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = match source {
            #[cfg(not(target_arch = "wasm32"))]
            None => device.create_shader_module(wgpu::include_spirv!(concat!(
                "..",
                path_separator!(),
                "shaders",
                path_separator!(),
                "spirv",
                path_separator!(),
                "accumulation.comp.spv"
            ))),
            #[cfg(target_arch = "wasm32")]
            None => device.create_shader_module(wgpu::include_spirv!(concat!(
                "..",
                path_separator!(),
                "shaders",
                path_separator!(),
                "spirv",
                path_separator!(),
                "accumulation_pingpong.comp.spv"
            ))),
            Some(v) => device.create_shader_module(v),
        };

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Accumulation Pipeline"),
            layout: Some(&pipeline_layout),
            entry_point: "main",
            module: &shader,
        });

        AccumulationPass {
            bind_group_layout,
            pipeline,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn create_frame_bind_groups(
        &self,
        device: &wgpu::Device,
        in_rays: &GPUBuffer<Ray>,
        global_uniforms: &UniformBuffer<PerDrawUniforms>,
        view: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Accumulation Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: Self::RAY_BINDING,
                    resource: in_rays.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: Self::TEXTURE_BINDING,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: Self::PER_DRAW_STRUCT_BINDING,
                    resource: global_uniforms.as_entire_binding(),
                },
            ],
        })
    }

    #[cfg(target_arch = "wasm32")]
    pub fn create_frame_bind_groups(
        &self,
        device: &wgpu::Device,
        in_rays: &GPUBuffer<Ray>,
        global_uniforms: &UniformBuffer<PerDrawUniforms>,
        write_view: &wgpu::TextureView,
        input_view: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Accumulation Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: Self::RAY_BINDING,
                    resource: in_rays.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: Self::TEXTURE_BINDING,
                    resource: wgpu::BindingResource::TextureView(write_view),
                },
                wgpu::BindGroupEntry {
                    binding: Self::PER_DRAW_STRUCT_BINDING,
                    resource: global_uniforms.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: Self::READ_TEXTURE_BINDING,
                    resource: wgpu::BindingResource::TextureView(input_view),
                },
                wgpu::BindGroupEntry {
                    binding: Self::SAMPLER_BINDING,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        })
    }

    pub fn dispatch(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        frame_bind_groups: &wgpu::BindGroup,
        dispatch_size: (u32, u32, u32),
    ) {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Accumulation Pass"),
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, frame_bind_groups, &[]);
        pass.dispatch_workgroups(dispatch_size.0, dispatch_size.1, dispatch_size.2);
    }
}
