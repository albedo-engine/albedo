use albedo_backend::{shader_bindings};
use crate::renderer::resources;

pub struct GPUIntersector {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
    pipeline: wgpu::Pipeline,
    bind_group: Option<wgpu::BindGroup>,
    count: u32,
}

impl GPUIntersector {

    pub fn new(device: &wgpu::Device) -> GPUIntersector {

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("GPUIntersector Layout"),
            entries: &[
                shader_bindings::buffer_entry(binding: 0, wgpu::ShaderStage::COMPUTE, true),
                shader_bindings::buffer_entry(binding: 1, wgpu::ShaderStage::COMPUTE, true),
                shader_bindings::buffer_entry(binding: 2, wgpu::ShaderStage::COMPUTE, true),
                shader_bindings::buffer_entry(binding: 3, wgpu::ShaderStage::COMPUTE, true),
                shader_bindings::buffer_entry(binding: 4, wgpu::ShaderStage::COMPUTE, true),
                shader_bindings::buffer_entry(binding: 5, wgpu::ShaderStage::COMPUTE, true),
                shader_bindings::buffer_entry(binding: 6, wgpu::ShaderStage::COMPUTE, false),
                shader_bindings::uniform_entry(binding: 7, wgpu::ShaderStage::COMPUTE),
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Intersector Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader =
            device.create_shader_module(&wgpu::include_spirv!("../shaders/intersection.comp.spv"));

        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Intersector Pipeline"),
            layout: Some(&pipeline_layout),
            entry_point: "main",
            module: &cs_module,
        });

        GPUIntersector {
            bind_group_layout,
            pipeline_layout,
            pipeline,
            bind_group: None,
            count: 0,
        }
    }

    pub fn bind_buffers(
        &mut self,
        out_intersections: &GPUBuffer<resources::IntersectionGPU>,
        instances: &GPUBuffer<resources::InstanceGPU>,
        nodes: &GPUBuffer<resources::BVHNodeGPU>,
        indices: &GPUBuffer<u32>,
        vertices: &GPUBuffer<resources::VertexGPU>,
        lights: &GPUBuffer<resources::LightGPU>,
        rays: &GPUBuffer<resources::RayGPU>,
        scene_info: &GPUBuffer<resources::SceneSettingsGPU>
    ) {
        self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Intersector Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: instances.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: nodes.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: indices.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: vertices.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: lights.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: rays.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: out_intersections.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: scene_info.as_entire_binding(),
                },
            ],
        });
        self.count = out_intersections.count();
    }

    pub fn run(&self, device: &Device, encoder: &mut CommandEncoder) {
        if let Some(bind_group) = self.bind_group {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Intersector Compute Pass")
            });
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);
            // @todo: how to deal with hardcoded size.
            compute_pass.dispatch(self.count / 8, 1, 1);
        }
    }

}
