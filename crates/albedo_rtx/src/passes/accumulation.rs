use crate::renderer::resources;
use albedo_backend::{shader_bindings, GPUBuffer, UniformBuffer};

pub struct AccumulationPass {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
    bind_group: Option<wgpu::BindGroup>,
}

impl AccumulationPass {
    pub fn new(device: &wgpu::Device) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Ray Generator Layout"),
            entries: &[
                shader_bindings::buffer_entry(0, wgpu::ShaderStage::COMPUTE, true),
                shader_bindings::texture2d_entry(1, wgpu::ShaderStage::COMPUTE),
                shader_bindings::uniform_entry(2, wgpu::ShaderStage::COMPUTE),
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Accumulation Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader =
            device.create_shader_module(&wgpu::include_spirv!("../shaders/accumulation.comp.spv"));

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Accumulation Pipeline"),
            layout: Some(&pipeline_layout),
            entry_point: "main",
            module: &shader,
        });

        AccumulationPass {
            bind_group: None,
            bind_group_layout,
            pipeline,
        }
    }

    pub fn bind(
        &mut self,
        device: &wgpu::Device,
        in_rays: &GPUBuffer<resources::RayGPU>,
        view: &wgpu::TextureView,
        global_uniforms: &UniformBuffer<resources::GlobalUniformsGPU>,
    ) {
        self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Accumulation Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: in_rays.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: global_uniforms.as_entire_binding(),
                },
            ],
        }));
    }

    pub fn run(&self, encoder: &mut wgpu::CommandEncoder, width: u32, height: u32) {
        match &self.bind_group {
            Some(bind_group) => {
                let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Accumulation Compute Pass"),
                });
                compute_pass.set_pipeline(&self.pipeline);
                compute_pass.set_bind_group(0, bind_group, &[]);
                // @todo: how to deal with hardcoded size.
                compute_pass.dispatch(width / 8, height / 8, 1);
            }
            _ => (),
        }
    }
}
