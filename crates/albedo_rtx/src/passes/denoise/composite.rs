use std::borrow::Cow;

use albedo_backend::data::{CompileError, PreprocessError, ShaderCache};

use crate::get_dispatch_size;
use crate::macros::path_separator;

use super::super::GBUFFER_READ_TY;

pub struct CompositingPass {
    pipeline: wgpu::ComputePipeline,
}

impl CompositingPass {
    const WORKGROUP_SIZE: (u32, u32, u32) = (8, 8, 1);
    const SHADER_ID: &'static str = "compositing.comp";

    const GBUFFER_BINDING: u32 = 0;
    const RADIANCE_BINDING: u32 = 1;
    const RADIANCE_OUT_BINDING: u32 = 2;
    const SAMPLER_BINDING: u32 = 3;

    pub fn new_inlined(device: &wgpu::Device, processor: &ShaderCache) -> Self {
        Self::new_raw(
            device,
            processor,
            include_str!(concat!(
                "..",
                path_separator!(),
                "..",
                path_separator!(),
                "..",
                path_separator!(),
                "shaders",
                path_separator!(),
                "compositing.comp"
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

    fn new_raw(
        device: &wgpu::Device,
        processor: &ShaderCache,
        src: &str,
    ) -> Result<Self, CompileError> {
        let frame_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Compositing Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: Self::GBUFFER_BINDING,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: GBUFFER_READ_TY,
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: Self::RADIANCE_BINDING,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: Self::RADIANCE_OUT_BINDING,
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
                ],
            });

        let pipeline_layout: wgpu::PipelineLayout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Composit Pipeline Layout"),
                bind_group_layouts: &[&frame_bind_group_layout],
                push_constant_ranges: &[],
            });

        let module = processor.compile_compute(src, None)?;
        let shader: wgpu::ShaderModule =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Composit Shader"),
                source: wgpu::ShaderSource::Naga(Cow::Owned(module)),
            });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Composit Pipeline"),
            layout: Some(&pipeline_layout),
            entry_point: Some("main"),
            module: &shader,
            compilation_options: Default::default(),
            cache: None,
        });

        Ok(Self { pipeline })
    }

    pub fn dispatch(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        bindgroup: &wgpu::BindGroup,
        size: &(u32, u32, u32),
    ) {
        let workgroups = get_dispatch_size(&size, &Self::WORKGROUP_SIZE);
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Compositing Pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, bindgroup, &[]);
        pass.dispatch_workgroups(workgroups.0, workgroups.1, workgroups.2);
    }
}
