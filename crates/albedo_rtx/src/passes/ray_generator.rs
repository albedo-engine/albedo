use albedo_backend::{shader_bindings, ComputePassDescriptor, GPUBuffer, UniformBuffer};

use crate::macros::path_separator;
use crate::renderer::resources;

pub struct RayGeneratorPassDescriptor {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
}

impl RayGeneratorPassDescriptor {
    pub fn new(device: &wgpu::Device) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Ray Generator Layout"),
            entries: &[
                shader_bindings::buffer_entry(0, wgpu::ShaderStages::COMPUTE, false),
                shader_bindings::uniform_entry(1, wgpu::ShaderStages::COMPUTE),
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Ray Generator Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(wgpu::include_spirv!(concat!(
            "..",
            path_separator!(),
            "shaders",
            path_separator!(),
            "spirv",
            path_separator!(),
            "ray_generation.comp.spv"
        )));

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Ray Generator Pipeline"),
            layout: Some(&pipeline_layout),
            entry_point: "main",
            module: &shader,
        });

        RayGeneratorPassDescriptor {
            bind_group_layout,
            pipeline,
        }
    }

    pub fn create_frame_bind_groups(
        &self,
        device: &wgpu::Device,
        out_rays: &GPUBuffer<resources::RayGPU>,
        camera: &UniformBuffer<resources::CameraGPU>,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Intersector Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: out_rays.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: camera.as_entire_binding(),
                },
            ],
        })
    }
}

impl ComputePassDescriptor for RayGeneratorPassDescriptor {
    type FrameBindGroups = wgpu::BindGroup;
    type PassBindGroups = ();

    fn get_name() -> &'static str {
        "Ray Generation Pass"
    }

    fn get_pipeline(&self) -> &wgpu::ComputePipeline {
        &self.pipeline
    }

    fn set_pass_bind_groups(_: &mut wgpu::ComputePass, _: &Self::PassBindGroups) {}

    fn set_frame_bind_groups<'a, 'b>(
        pass: &mut wgpu::ComputePass<'a>,
        groups: &'b Self::FrameBindGroups,
    ) where
        'b: 'a,
    {
        pass.set_bind_group(0, groups, &[]);
    }
}
