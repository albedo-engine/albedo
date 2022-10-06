use crate::macros::path_separator;
use crate::renderer::resources;
use albedo_backend::{shader_bindings, GPUBuffer, UniformBuffer};

pub struct BVHDebugPass {
    bind_group_layouts: [wgpu::BindGroupLayout; 1],
    pipeline: wgpu::ComputePipeline,
    base_bind_group: Option<wgpu::BindGroup>,
}

impl BVHDebugPass {
    pub fn new(device: &wgpu::Device) -> Self {
        let bind_group_layouts =
            [
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Debug BVH Base Layout"),
                    entries: &[
                        shader_bindings::buffer_entry(0, wgpu::ShaderStages::COMPUTE, false),
                        shader_bindings::buffer_entry(1, wgpu::ShaderStages::COMPUTE, true),
                        shader_bindings::buffer_entry(2, wgpu::ShaderStages::COMPUTE, true),
                        shader_bindings::buffer_entry(3, wgpu::ShaderStages::COMPUTE, true),
                        shader_bindings::buffer_entry(4, wgpu::ShaderStages::COMPUTE, true),
                        shader_bindings::uniform_entry(5, wgpu::ShaderStages::COMPUTE),
                    ],
                }),
            ];

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Debug BVH Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layouts[0]],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(wgpu::include_spirv!(concat!(
            "..",
            path_separator!(),
            "shaders",
            path_separator!(),
            "spirv",
            path_separator!(),
            "debug_bvh.comp.spv"
        )));

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Debug BVH Pipeline"),
            layout: Some(&pipeline_layout),
            entry_point: "main",
            module: &shader,
        });

        BVHDebugPass {
            bind_group_layouts,
            pipeline,
            base_bind_group: None,
        }
    }

    pub fn bind_buffers(
        &mut self,
        device: &wgpu::Device,
        out_rays: &GPUBuffer<resources::RayGPU>,
        instances: &GPUBuffer<resources::InstanceGPU>,
        nodes: &wgpu::Buffer,
        indices: &GPUBuffer<u32>,
        vertices: &GPUBuffer<resources::VertexGPU>,
    ) {
        self.base_bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Debug BVH Base Bind Group"),
            layout: &self.bind_group_layouts[0],
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: out_rays.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: instances.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: nodes.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: indices.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: vertices.as_entire_binding(),
                },
            ],
        }));
    }

    pub fn run(&self, encoder: &mut wgpu::CommandEncoder, width: u32, height: u32) {
        match &self.base_bind_group {
            Some(base_group) => {
                let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Debug BVH Compute Pass"),
                });
                compute_pass.set_pipeline(&self.pipeline);
                compute_pass.set_bind_group(0, base_group, &[]);
                // @todo: how to deal with hardcoded size.
                compute_pass.dispatch_workgroups(width / 8, height / 8, 1);
            }
            _ => (),
        }
    }
}
