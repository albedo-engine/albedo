use albedo_backend::{GPUBuffer, UniformBuffer};

use crate::macros::path_separator;
use crate::uniforms;

pub struct RayPass {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
}

/// Ray generation passs.
///
/// This pass fills a buffer of [`uniforms::Ray`] structures based
/// on the camera information.
impl RayPass {
    const RAY_BINDING: u32 = 0;
    const CAMERA_BINDING: u32 = 1;

    pub fn new(device: &wgpu::Device, source: Option<crate::passes::ShaderSource<()>>) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Ray Generator Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: Self::RAY_BINDING,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: Self::CAMERA_BINDING,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Ray Generator Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let shader = match source {
            None => device.create_shader_module(wgpu::include_spirv!(concat!(
                "..",
                path_separator!(),
                "shaders",
                path_separator!(),
                "spirv",
                path_separator!(),
                "ray_generation.comp.spv"
            ))),
            Some(v) => device.create_shader_module(v.descriptor),
        };
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Ray Generator Pipeline"),
            layout: Some(&pipeline_layout),
            entry_point: "main",
            module: &shader,
        });
        Self {
            bind_group_layout,
            pipeline,
        }
    }

    pub fn create_frame_bind_groups(
        &self,
        device: &wgpu::Device,
        out_rays: &GPUBuffer<uniforms::Ray>,
        camera: &UniformBuffer<uniforms::Camera>,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Ray Generation Frame Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: Self::RAY_BINDING,
                    resource: out_rays.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: Self::CAMERA_BINDING,
                    resource: camera.as_entire_binding(),
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
            label: Some("Ray Generator Pass"),
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, frame_bind_groups, &[]);
        pass.dispatch_workgroups(dispatch_size.0, dispatch_size.1, dispatch_size.2);
    }
}