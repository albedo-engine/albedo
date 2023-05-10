use crate::get_dispatch_size;
use crate::macros::path_separator;
use crate::uniforms;
use albedo_backend::gpu;

pub struct AOPass {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
}

impl AOPass {
    const WORKGROUP_SIZE: (u32, u32, u32) = (8, 8, 1);

    const RAY_BINDING: u32 = 0;
    const NODE_BINDING: u32 = 1;
    const INTERSECTION_BINDING: u32 = 2;
    const INSTANCES_BINDING: u32 = 3;
    const INDEX_BINDING: u32 = 4;
    const VERTEX_BINDING: u32 = 5;
    const PER_DRAW_STRUCT_BINDING: u32 = 6;

    pub fn new(device: &wgpu::Device, source: Option<wgpu::ShaderModuleDescriptor>) -> Self {
        let bind_group_layout = gpu::BindGroupLayoutBuilder::new_with_size(6)
            .label(Some("Accumulation Bind Group Layout"))
            .storage_buffer(wgpu::ShaderStages::COMPUTE, false)
            .storage_buffer(wgpu::ShaderStages::COMPUTE, true)
            .storage_buffer(wgpu::ShaderStages::COMPUTE, true)
            .storage_buffer(wgpu::ShaderStages::COMPUTE, true)
            .storage_buffer(wgpu::ShaderStages::COMPUTE, true)
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

    pub fn create_frame_bind_groups(
        &self,
        device: &wgpu::Device,
        in_rays: gpu::StorageBufferSlice<uniforms::Ray>,
        in_nodes: &wgpu::Buffer,
        in_intersections: gpu::StorageBufferSlice<uniforms::Intersection>,
        in_instances: gpu::StorageBufferSlice<uniforms::Instance>,
        in_indices: gpu::StorageBufferSlice<u32>,
        in_vertices: &wgpu::Buffer,
        global_uniforms: gpu::UniformBufferSlice<uniforms::PerDrawUniforms>,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("AO Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: Self::RAY_BINDING,
                    resource: in_rays.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: Self::NODE_BINDING,
                    resource: in_nodes.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: Self::INTERSECTION_BINDING,
                    resource: in_intersections.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: Self::INSTANCES_BINDING,
                    resource: in_instances.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: Self::INDEX_BINDING,
                    resource: in_indices.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: Self::VERTEX_BINDING,
                    resource: in_vertices.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: Self::PER_DRAW_STRUCT_BINDING,
                    resource: global_uniforms.as_entire_binding(),
                },
            ],
        })
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
