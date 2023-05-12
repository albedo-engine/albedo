use crate::{get_dispatch_size, SceneLayout};
use crate::macros::path_separator;
use crate::uniforms::{Instance, PerDrawUniforms, Ray, Intersection};
use albedo_backend::gpu;

pub struct AOPass {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
}

impl AOPass {
    const WORKGROUP_SIZE: (u32, u32, u32) = (8, 8, 1);

    const RAY_BINDING: u32 = 0;
    const INTERSECTION_BINDING: u32 = 2;

    pub fn new(device: &wgpu::Device, source: Option<wgpu::ShaderModuleDescriptor>) -> Self {
        let bind_group_layout = gpu::BindGroupLayoutBuilder::new_with_size(6)
            .label(Some("Accumulation Bind Group Layout"))
            .storage_buffer(wgpu::ShaderStages::COMPUTE, false)
            .storage_buffer(wgpu::ShaderStages::COMPUTE, true)
            .uniform_buffer(wgpu::ShaderStages::COMPUTE, None)
            .build(device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Accumulation Pipeline Layout"),
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
                "ao.comp.spv"
            ))),
            Some(v) => device.create_shader_module(v),
        };

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("AO Pipeline"),
            layout: Some(&pipeline_layout),
            entry_point: "main",
            module: &shader,
        });

        AOPass {
            bind_group_layout,
            pipeline,
        }
    }

    pub fn create_bind_groups(
        &self,
        device: &wgpu::Device,
        scene_layout: &SceneLayout,
        instances: &gpu::Buffer<Instance>,
        nodes: &wgpu::Buffer,
        indices: &gpu::Buffer<u32>,
        vertices: &wgpu::Buffer,
        rays: gpu::Buffer<Ray>,
        intersections: gpu::Buffer<Intersection>,
        global_uniforms: &gpu::Buffer<PerDrawUniforms>,
    ) -> [wgpu::BindGroup; 2] {
        let scene_bind_group = scene_layout.scene_bind_group(1).uniforms(global_uniforms).create(device);
        let geometry_bind_group = scene_layout.geometry_bind_group(4).instances(instances).nodes(nodes).indices(indices).vertices(vertices).create(device);
        let shading_bind_group = scene_layout.shading_bind_group(0).create(device);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("AO Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: Self::RAY_BINDING,
                    resource: rays.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: Self::INTERSECTION_BINDING,
                    resource: intersections.as_entire_binding(),
                },
            ],
        });
        [scene_bind_group, geometry_bind_group, shading_bind_group, bind_group]
    }

    pub fn dispatch(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        frame_bind_groups: &wgpu::BindGroup,
        size: (u32, u32, u32),
    ) {
        let mut pass: wgpu::ComputePass =
            encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("AO Pass"),
            });
        let workgroups = get_dispatch_size(size, Self::WORKGROUP_SIZE);
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, frame_bind_groups, &[]);
        pass.dispatch_workgroups(workgroups.0, workgroups.1, workgroups.2);
    }
}
