use std::borrow::Cow;

use albedo_backend::data::{CompileError, PreprocessError, ShaderCache};
use albedo_backend::gpu;

use crate::macros::path_separator;
use crate::{get_dispatch_size, uniforms};

use super::GBUFFER_READ_TY;

pub struct TemporalAccumulationPass {
    frame_bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
}

impl TemporalAccumulationPass {
    const WORKGROUP_SIZE: (u32, u32, u32) = (8, 8, 1);
    const SHADER_ID: &'static str = "temporal-accumulation.comp";

    const RAYS_BINDING: u32 = 0;
    const GBUFFER_PREVIOUS_BINDING: u32 = 1;
    const GBUFFER_BINDING: u32 = 2;
    const MOTION_BINDING: u32 = 3;
    const RADIANCE_PREVIOUS_BINDING: u32 = 4;
    const RADIANCE_BINDING: u32 = 5;
    const SAMPLER_BINDING: u32 = 6;
    const HISTORY_PREVIOUS_BINDING: u32 = 7;
    const HISTORY_BINDING: u32 = 8;
    const MOMENTS_PREVIOUS_BINDING: u32 = 9;
    const MOMENTS_BINDING: u32 = 10;

    pub fn new_inlined(device: &wgpu::Device, processor: &ShaderCache) -> Self {
        Self::new_raw(
            device,
            processor,
            include_str!(concat!(
                "..",
                path_separator!(),
                "..",
                path_separator!(),
                "shaders",
                path_separator!(),
                "temporal-accumulation.comp"
            )),
        )
        .unwrap()
    }

    pub fn new(device: &wgpu::Device, processor: &ShaderCache) -> Result<Self, CompileError> {
        let Some(source) = processor.get(Self::SHADER_ID) else {
            return Err(PreprocessError::Missing(Self::SHADER_ID.to_string()).into());
        };
        Self::new_raw(device, processor, source)
    }

    pub fn new_raw(
        device: &wgpu::Device,
        processor: &ShaderCache,
        source: &str,
    ) -> Result<Self, CompileError> {
        let frame_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Temporal Accumulation Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: Self::RAYS_BINDING,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: Self::GBUFFER_PREVIOUS_BINDING,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: GBUFFER_READ_TY,
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: Self::GBUFFER_BINDING,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: GBUFFER_READ_TY,
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: Self::MOTION_BINDING,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: Self::RADIANCE_PREVIOUS_BINDING,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: Self::RADIANCE_BINDING,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            format: wgpu::TextureFormat::Rgba32Float,
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: Self::SAMPLER_BINDING,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: Self::HISTORY_PREVIOUS_BINDING,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: Self::HISTORY_BINDING,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: Self::MOMENTS_PREVIOUS_BINDING,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: Self::MOMENTS_BINDING,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            format: wgpu::TextureFormat::Rg32Float,
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                ],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Temporal Accumulation Pipeline Layout"),
            bind_group_layouts: &[&frame_bind_group_layout],
            push_constant_ranges: &[],
        });

        let module = processor.compile_compute(source, None)?;
        let shader: wgpu::ShaderModule =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Temporal Accumulation Shader"),
                source: wgpu::ShaderSource::Naga(Cow::Owned(module)),
            });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Temporal Accumulation Pipeline"),
            layout: Some(&pipeline_layout),
            entry_point: Some("main"),
            module: &shader,
            compilation_options: Default::default(),
            cache: None,
        });

        Ok(Self {
            frame_bind_group_layout,
            pipeline,
        })
    }

    pub fn create_frame_bind_groups(
        &self,
        device: &wgpu::Device,
        out_radiance: &wgpu::TextureView,
        out_moments: &wgpu::TextureView,
        out_history: &gpu::Buffer<u32>,
        rays: &gpu::Buffer<uniforms::Ray>,
        gbuffer_previous: &wgpu::TextureView,
        gbuffer: &wgpu::TextureView,
        motion: &wgpu::TextureView,
        radiance_previous: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
        history_previous: &gpu::Buffer<u32>,
        moments_previous: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Temporal Accumulation Frame Bind Group"),
            layout: &self.frame_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: Self::RAYS_BINDING,
                    resource: rays.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: Self::GBUFFER_PREVIOUS_BINDING,
                    resource: wgpu::BindingResource::TextureView(gbuffer_previous),
                },
                wgpu::BindGroupEntry {
                    binding: Self::GBUFFER_BINDING,
                    resource: wgpu::BindingResource::TextureView(gbuffer),
                },
                wgpu::BindGroupEntry {
                    binding: Self::MOTION_BINDING,
                    resource: wgpu::BindingResource::TextureView(motion),
                },
                wgpu::BindGroupEntry {
                    binding: Self::RADIANCE_PREVIOUS_BINDING,
                    resource: wgpu::BindingResource::TextureView(radiance_previous),
                },
                wgpu::BindGroupEntry {
                    binding: Self::RADIANCE_BINDING,
                    resource: wgpu::BindingResource::TextureView(out_radiance),
                },
                wgpu::BindGroupEntry {
                    binding: Self::SAMPLER_BINDING,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
                wgpu::BindGroupEntry {
                    binding: Self::HISTORY_PREVIOUS_BINDING,
                    resource: history_previous.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: Self::HISTORY_BINDING,
                    resource: out_history.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: Self::MOMENTS_PREVIOUS_BINDING,
                    resource: wgpu::BindingResource::TextureView(moments_previous),
                },
                wgpu::BindGroupEntry {
                    binding: Self::MOMENTS_BINDING,
                    resource: wgpu::BindingResource::TextureView(out_moments),
                },
            ],
        })
    }

    pub fn dispatch(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        frame_bind_group: &wgpu::BindGroup,
        size: &(u32, u32, u32),
    ) {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Temporal Accumulation Pass"),
            timestamp_writes: None,
        });
        let workgroups = get_dispatch_size(&size, &Self::WORKGROUP_SIZE);
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, frame_bind_group, &[]);
        pass.dispatch_workgroups(workgroups.0, workgroups.1, workgroups.2);
    }
}
