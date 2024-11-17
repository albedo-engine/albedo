use std::borrow::{Borrow, Cow};

use albedo_backend::data::ShaderCache;
use albedo_backend::gpu;
use wgpu::naga::{self, FastHashMap};

use crate::get_dispatch_size;
use crate::macros::path_separator;
use crate::uniforms;

pub struct RayPass {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
    pipeline: wgpu::ComputePipeline,
}

/// Ray generation passs.
///
/// This pass fills a buffer of [`uniforms::Ray`] structures based
/// on the camera information.
impl RayPass {
    const RAY_BINDING: u32 = 0;
    const CAMERA_BINDING: u32 = 1;

    const WORKGROUP_SIZE: (u32, u32, u32) = (8, 8, 1);

    pub fn new(device: &wgpu::Device, processor: &ShaderCache, source: Option<&str>) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Ray Generator Layout"),
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
                    binding: Self::CAMERA_BINDING,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Ray Generator Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let module = processor.compile_compute(source.unwrap_or(include_str!(concat!(
            "..",
            path_separator!(),
            "..",
            path_separator!(),
            "shaders",
            path_separator!(),
            "ray_generation.comp"
        )))).unwrap();

        let shader: wgpu::ShaderModule = device.create_shader_module(wgpu::ShaderModuleDescriptor{
            label: Some("Ray Generation Shader"),
            source: wgpu::ShaderSource::Naga(Cow::Owned(module))
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Ray Generator Pipeline"),
            layout: Some(&pipeline_layout),
            entry_point: Some("main"),
            module: &shader,
            compilation_options: Default::default(),
            cache: None,
        });
        Self {
            bind_group_layout,
            pipeline_layout,
            pipeline,
        }
    }

    pub fn set_shader(&mut self, device: &wgpu::Device, shader: wgpu::ShaderModuleDescriptor) {
        let shader = device.create_shader_module(shader);
        self.pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Ray Generator Pipeline"),
            layout: Some(&self.pipeline_layout),
            entry_point: Some("main"),
            module: &shader,
            compilation_options: Default::default(),
            cache: None,
        });
    }

    pub fn create_frame_bind_groups(
        &self,
        device: &wgpu::Device,
        out_rays: &gpu::Buffer<uniforms::Ray>,
        camera: &gpu::UniformBufferSlice<uniforms::Camera>,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Ray Generation Frame Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: Self::RAY_BINDING,
                    resource: out_rays.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: Self::CAMERA_BINDING,
                    resource: camera.as_entire_binding(),
                },
            ],
        })
    }

    pub fn dispatch(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        frame_bind_groups: &wgpu::BindGroup,
        dispatch_size: (u32, u32, u32),
    ) {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Ray Generator Pass"),
            timestamp_writes: None,
        });
        let workgroups = get_dispatch_size(&dispatch_size, &Self::WORKGROUP_SIZE);
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, frame_bind_groups, &[]);
        pass.dispatch_workgroups(workgroups.0, workgroups.1, workgroups.2);
    }
}
