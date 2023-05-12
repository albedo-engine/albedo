use albedo_backend::gpu;

use crate::macros::path_separator;
use crate::uniforms;
use crate::{SceneLayout};

pub struct IntersectorPass {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
}

impl IntersectorPass {
    const RAY_BINDING: u32 = 5;
    const INTERSECTION_BINDING: u32 = 6;

    pub fn new(device: &wgpu::Device, scene_layout: &SceneLayout, source: Option<wgpu::ShaderModuleDescriptor>) -> Self {
        let scene_layout = &scene_layout.layout;
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Intersector Bind Group Layout"),
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
                    binding: Self::INTERSECTION_BINDING,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Intersector Pipeline Layout"),
            bind_group_layouts: &[&scene_layout, &bind_group_layout],
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
                "intersection.comp.spv"
            ))),
            Some(v) => device.create_shader_module(v),
        };

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Intersector Pipeline"),
            layout: Some(&pipeline_layout),
            entry_point: "main",
            module: &shader,
        });

        IntersectorPass {
            bind_group_layout,
            pipeline,
        }
    }

    pub fn create_bind_groups(
        &self,
        device: &wgpu::Device,
        scene_layout: &SceneLayout,
        out_intersections: &gpu::Buffer<uniforms::Intersection>,
        instances: &gpu::Buffer<uniforms::Instance>,
        nodes: &wgpu::Buffer,
        indices: &gpu::Buffer<u32>,
        vertices: &wgpu::Buffer,
        lights: &gpu::Buffer<uniforms::Light>,
        rays: &gpu::Buffer<uniforms::Ray>,
    ) -> [wgpu::BindGroup; 2] {
        let scene_bind_group = scene_layout.bind_group(0)
            .instances(instances).nodes(nodes).indices(indices).vertices(vertices).lights(lights).create(device);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Intersector Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: Self::RAY_BINDING,
                    resource: rays.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: Self::INTERSECTION_BINDING,
                    resource: out_intersections.as_entire_binding(),
                },
            ],
        });
        [scene_bind_group, bind_group]
    }

    pub fn dispatch(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        bind_groups: &[wgpu::BindGroup; 2],
        dispatch_size: (u32, u32, u32),
    ) {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Intersector Pass"),
        });
        pass.set_pipeline(&self.pipeline);
        for i in 0..bind_groups.len() {
            pass.set_bind_group(i as u32, &bind_groups[i], &[]);
        }
        pass.dispatch_workgroups(dispatch_size.0, dispatch_size.1, dispatch_size.2);
    }
}
