use crate::renderer::resources;
use albedo_backend::{shader_bindings, GPUBuffer, UniformBuffer};

pub struct GPURayGenerator {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
    pipeline: wgpu::ComputePipeline,
    bind_group: Option<wgpu::BindGroup>,
}

impl GPURayGenerator {
    pub fn new(device: &wgpu::Device) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Ray Generator Layout"),
            entries: &[
                shader_bindings::buffer_entry(0, wgpu::ShaderStage::COMPUTE, false),
                shader_bindings::uniform_entry(1, wgpu::ShaderStage::COMPUTE),
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Ray Generator Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = device
            .create_shader_module(&wgpu::include_spirv!("../shaders/ray_generation.comp.spv"));

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Ray Generator Pipeline"),
            layout: Some(&pipeline_layout),
            entry_point: "main",
            module: &shader,
        });

        GPURayGenerator {
            bind_group_layout,
            pipeline_layout,
            pipeline,
            bind_group: None,
        }
    }

    pub fn bind_buffers(
        &mut self,
        device: &wgpu::Device,
        out_rays: &GPUBuffer<resources::RayGPU>,
        camera: &UniformBuffer<resources::CameraGPU>,
    ) {
        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
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
        }));
    }

    pub fn run(&self, encoder: &mut wgpu::CommandEncoder, width: u32, height: u32) {
        if let Some(bind_group) = &self.bind_group {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Ray Generator Compute Pass"),
            });
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, bind_group, &[]);
            // @todo: how to deal with hardcoded size work size.
            compute_pass.dispatch(width / 8, height / 8, 1);
        }
    }
}
