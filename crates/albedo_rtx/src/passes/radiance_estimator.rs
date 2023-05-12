use albedo_backend::gpu;

use crate::get_dispatch_size;
use crate::macros::path_separator;
use crate::uniforms;
use crate::{SceneLayout};

pub struct ShadingPass {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
    pipeline: wgpu::ComputePipeline,
}

impl ShadingPass {
    const WORKGROUP_SIZE: (u32, u32, u32) = (8, 8, 1);

    const RAY_BINDING: u32 = 0;
    const INTERSECTION_BINDING: u32 = 2;

    pub fn new(device: &wgpu::Device, scene_layout: &SceneLayout) -> Self {
        let scene_layout = &scene_layout.layout;
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Radiance Estimator Base Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: Self::RAY_BINDING,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: Self::INTERSECTION_BINDING,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Radiance Estimator Pipeline Layout"),
            bind_group_layouts: &[&scene_layout, &bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(wgpu::include_spirv!(concat!(
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

        Self {
            bind_group_layout,
            pipeline_layout,
            pipeline,
        }
    }

    pub fn set_shader(&mut self, device: &wgpu::Device, shader_desc: wgpu::ShaderModuleDescriptor) {
        let shader = device.create_shader_module(shader_desc);
        self.pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Radiance Estimator Pipeline"),
            layout: Some(&self.pipeline_layout),
            entry_point: "main",
            module: &shader,
        });
    }

    pub fn create_bind_groups(
        &self,
        device: &wgpu::Device,
        scene_layout: &SceneLayout,
        out_rays: &gpu::Buffer<uniforms::Ray>,
        intersections: &gpu::Buffer<uniforms::Intersection>,
        instances: &gpu::Buffer<uniforms::Instance>,
        nodes: &wgpu::Buffer,
        indices: &gpu::Buffer<u32>,
        vertices: &wgpu::Buffer,
        lights: &gpu::Buffer<uniforms::Light>,
        materials: &gpu::Buffer<uniforms::Material>,
        probe_view: &wgpu::TextureView,
        texture_info: &wgpu::TextureView,
        atlas_view: &wgpu::TextureView,
        global_uniforms: &gpu::UniformBufferSlice<uniforms::PerDrawUniforms>,
        sampler_nearest: &wgpu::Sampler,
        sampler_linear: &wgpu::Sampler,
    ) -> [wgpu::BindGroup; 2] {
        let scene_bind_group = scene_layout.bind_group(1)
            .uniforms(global_uniforms)
            .instances(instances).nodes(nodes).indices(indices).vertices(vertices)
            .lights(lights).materials(materials).textures(probe_view, texture_info, atlas_view, sampler_nearest, sampler_linear)
            .create(device);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Radiance Estimator Base Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: Self::RAY_BINDING,
                    resource: out_rays.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: Self::INTERSECTION_BINDING,
                    resource: intersections.as_entire_binding(),
                },
            ],
        });
        [scene_bind_group, bind_group]
    }

    pub fn dispatch(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        bind_groups: &[wgpu::BindGroup; 2],
        size: (u32, u32, u32),
    ) {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Shading Pass"),
        });
        let workgroups = get_dispatch_size(size, Self::WORKGROUP_SIZE);
        pass.set_pipeline(&self.pipeline);
        for i in 0..bind_groups.len() {
            pass.set_bind_group(i as u32, &bind_groups[i], &[]);
        }
        pass.dispatch_workgroups(workgroups.0, workgroups.1, workgroups.2);
    }
}
