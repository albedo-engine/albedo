use crate::macros::path_separator;
use crate::renderer::resources;
use albedo_backend::{shader_bindings, ComputePassDescriptor, GPUBuffer, UniformBuffer};

pub struct ShadingPassDescriptor {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
    pipeline: wgpu::ComputePipeline,
}

impl ShadingPassDescriptor {
    pub fn new(device: &wgpu::Device) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Radiance Estimator Base Layout"),
            entries: &[
                shader_bindings::buffer_entry(0, wgpu::ShaderStages::COMPUTE, false),
                shader_bindings::buffer_entry(1, wgpu::ShaderStages::COMPUTE, true),
                shader_bindings::buffer_entry(2, wgpu::ShaderStages::COMPUTE, true),
                shader_bindings::buffer_entry(3, wgpu::ShaderStages::COMPUTE, true),
                shader_bindings::buffer_entry(4, wgpu::ShaderStages::COMPUTE, true),
                shader_bindings::buffer_entry(5, wgpu::ShaderStages::COMPUTE, true),
                shader_bindings::buffer_entry(6, wgpu::ShaderStages::COMPUTE, true),
                shader_bindings::buffer_entry(7, wgpu::ShaderStages::COMPUTE, true),
                shader_bindings::texture2d_entry(8, wgpu::ShaderStages::COMPUTE),
                shader_bindings::texture1D_u(9, wgpu::ShaderStages::COMPUTE),
                shader_bindings::texture2darray_entry(10, wgpu::ShaderStages::COMPUTE),
                shader_bindings::sampler_entry(11, wgpu::ShaderStages::COMPUTE, true),
                shader_bindings::uniform_entry(12, wgpu::ShaderStages::COMPUTE),
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Radiance Estimator Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(&wgpu::include_spirv!(concat!(
            "..",
            path_separator!(),
            "shaders",
            path_separator!(),
            "spirv",
            path_separator!(),
            "shading.comp.spv"
        )));

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Radiance Estimator Pipeline"),
            layout: Some(&pipeline_layout),
            entry_point: "main",
            module: &shader,
        });

        ShadingPassDescriptor {
            bind_group_layout,
            pipeline_layout,
            pipeline,
        }
    }

    pub fn set_shader(
        &mut self,
        device: &wgpu::Device,
        shader_desc: &wgpu::ShaderModuleDescriptor,
    ) {
        let shader = device.create_shader_module(shader_desc);
        self.pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Radiance Estimator Pipeline"),
            layout: Some(&self.pipeline_layout),
            entry_point: "main",
            module: &shader,
        });
    }

    pub fn create_frame_bind_groups(
        &self,
        device: &wgpu::Device,
        out_rays: &GPUBuffer<resources::RayGPU>,
        nodes: &GPUBuffer<resources::BVHNodeGPU>,
        intersections: &GPUBuffer<resources::IntersectionGPU>,
        instances: &GPUBuffer<resources::InstanceGPU>,
        indices: &GPUBuffer<u32>,
        vertices: &GPUBuffer<resources::VertexGPU>,
        lights: &GPUBuffer<resources::LightGPU>,
        materials: &GPUBuffer<resources::MaterialGPU>,
        probe_view: &wgpu::TextureView,
        texture_info: &wgpu::TextureView,
        atlas_view: &wgpu::TextureView,
        global_uniforms: &UniformBuffer<resources::GlobalUniformsGPU>,
        sampler_nearest: &wgpu::Sampler,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Radiance Estimator Base Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: out_rays.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: nodes.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: intersections.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: instances.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: indices.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: vertices.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: lights.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: materials.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: wgpu::BindingResource::TextureView(probe_view),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: wgpu::BindingResource::TextureView(texture_info),
                },
                wgpu::BindGroupEntry {
                    binding: 10,
                    resource: wgpu::BindingResource::TextureView(atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 11,
                    resource: wgpu::BindingResource::Sampler(sampler_nearest),
                },
                wgpu::BindGroupEntry {
                    binding: 12,
                    resource: global_uniforms.as_entire_binding(),
                },
            ],
        })
    }
}

impl ComputePassDescriptor for ShadingPassDescriptor {
    type FrameBindGroups = wgpu::BindGroup;
    type PassBindGroups = ();

    fn get_name() -> &'static str {
        "Shading Pass"
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
