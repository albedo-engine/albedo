use std::borrow::{Borrow, Cow};

use albedo_backend::data::ShaderCache;
use albedo_backend::gpu;
use wgpu::naga;

use crate::macros::path_separator;
use crate::{get_dispatch_size, uniforms};

use super::GBUFFER_WRITE_TY;

pub struct GBufferPass {
    frame_bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
}

impl GBufferPass {
    const WORKGROUP_SIZE: (u32, u32, u32) = (8, 8, 1);

    const INTERSECTION_BINDING: u32 = 0;
    const GBUFFER_BINDING: u32 = 1;
    const MOTION_BINDING: u32 = 2;

    pub fn new(
        device: &wgpu::Device,
        processor: &ShaderCache,
        geometry_layout: &crate::RTGeometryBindGroupLayout,
        source: Option<&str>,
    ) -> Self {
        let frame_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("GBuffer Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: Self::INTERSECTION_BINDING,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: Self::GBUFFER_BINDING,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: GBUFFER_WRITE_TY,
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: Self::MOTION_BINDING,
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
            label: Some("GBuffer Pipeline Layout"),
            bind_group_layouts: &[geometry_layout, &frame_bind_group_layout],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::COMPUTE,
                range: 0..64,
            }],
        });

        let source = source.unwrap_or(include_str!(concat!(
            "..",
            path_separator!(),
            "..",
            path_separator!(),
            "shaders",
            path_separator!(),
            "gbuffer.comp"
        )));
        let source = processor.compile(source).unwrap();

        let shader: wgpu::ShaderModule = device.create_shader_module(wgpu::ShaderModuleDescriptor{
            label: Some("GBuffer Shader"),
            source: wgpu::ShaderSource::Glsl {
                shader: Cow::Borrowed(source.borrow()),
                stage: naga::ShaderStage::Compute,
                defines: naga::FastHashMap::default()
            }
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Gbuffer Pipeline"),
            layout: Some(&pipeline_layout),
            entry_point: Some("main"),
            module: &shader,
            compilation_options: Default::default(),
            cache: None,
        });

        GBufferPass {
            frame_bind_group_layout,
            pipeline,
        }
    }

    pub fn create_frame_bind_groups(
        &self,
        device: &wgpu::Device,
        out_gbuffer: &wgpu::TextureView,
        out_motion: &wgpu::TextureView,
        intersections: &gpu::Buffer<uniforms::Intersection>
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Gbuffer Frame Bind Group"),
            layout: &self.frame_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: Self::INTERSECTION_BINDING,
                    resource: intersections.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: Self::GBUFFER_BINDING,
                    resource: wgpu::BindingResource::TextureView(out_gbuffer),
                },
                wgpu::BindGroupEntry {
                    binding: Self::MOTION_BINDING,
                    resource: wgpu::BindingResource::TextureView(out_motion),
                },
            ],
        })
    }

    pub fn dispatch(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        scene_bind_group: &wgpu::BindGroup,
        frame_bind_group: &wgpu::BindGroup,
        size: &(u32, u32, u32),
        world_to_screen: &glam::Mat4 // @todo: Better to not use GLAM probably here
    ) {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("GBuffer Pass"),
            timestamp_writes: None,
        });
        let workgroups = get_dispatch_size(&size, &Self::WORKGROUP_SIZE);
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, scene_bind_group, &[]);
        pass.set_bind_group(1, frame_bind_group, &[]);
        {
            let data: &[f32; 16] = world_to_screen.as_ref();
            let data = bytemuck::cast_slice(data);
            pass.set_push_constants(0, data);
        }
        pass.dispatch_workgroups(workgroups.0, workgroups.1, workgroups.2);
    }
}
