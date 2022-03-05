use crate::renderer::resources;
use albedo_backend::{shader_bindings, ComputePassDescriptor, GPUBuffer, UniformBuffer};

pub struct IntersectorPassDescriptor {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
}

impl IntersectorPassDescriptor {
    pub fn new(device: &wgpu::Device) -> IntersectorPassDescriptor {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("GPUIntersector Layout"),
            entries: &[
                shader_bindings::buffer_entry(0, wgpu::ShaderStages::COMPUTE, true),
                shader_bindings::buffer_entry(1, wgpu::ShaderStages::COMPUTE, true),
                shader_bindings::buffer_entry(2, wgpu::ShaderStages::COMPUTE, true),
                shader_bindings::buffer_entry(3, wgpu::ShaderStages::COMPUTE, true),
                shader_bindings::buffer_entry(4, wgpu::ShaderStages::COMPUTE, true),
                shader_bindings::buffer_entry(5, wgpu::ShaderStages::COMPUTE, true),
                shader_bindings::buffer_entry(6, wgpu::ShaderStages::COMPUTE, false),
                shader_bindings::uniform_entry(7, wgpu::ShaderStages::COMPUTE),
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Intersector Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader =
            device.create_shader_module(&wgpu::include_spirv!("../shaders/intersection.comp.spv"));

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Intersector Pipeline"),
            layout: Some(&pipeline_layout),
            entry_point: "main",
            module: &shader,
        });

        IntersectorPassDescriptor {
            bind_group_layout,
            pipeline,
        }
    }

    pub fn create_frame_bind_groups(
        &self,
        device: &wgpu::Device,
        out_intersections: &GPUBuffer<resources::IntersectionGPU>,
        instances: &GPUBuffer<resources::InstanceGPU>,
        nodes: &GPUBuffer<resources::BVHNodeGPU>,
        indices: &GPUBuffer<u32>,
        vertices: &GPUBuffer<resources::VertexGPU>,
        lights: &GPUBuffer<resources::LightGPU>,
        rays: &GPUBuffer<resources::RayGPU>,
        scene_info: &UniformBuffer<resources::SceneSettingsGPU>,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
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
        })
    }
}

impl ComputePassDescriptor for IntersectorPassDescriptor {
    type FrameBindGroups = wgpu::BindGroup;
    type PassBindGroups = ();

    fn get_name() -> &'static str { "Intersection Pass" }

    fn get_pipeline(&self) -> &wgpu::ComputePipeline { &self.pipeline }

    fn set_pass_bind_groups(_: &mut wgpu::ComputePass, _: &Self::PassBindGroups) {}

    fn set_frame_bind_groups<'a, 'b>(pass: &mut wgpu::ComputePass<'a>, groups: &'b Self::FrameBindGroups)
        where 'b: 'a {
        pass.set_bind_group(0, groups, &[]);
    }
}
