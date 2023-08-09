use albedo_backend::gpu;

use crate::macros::path_separator;
use crate::uniforms;

pub struct IntersectorPass {
    frame_bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
}

impl IntersectorPass {
    const RAY_BINDING: u32 = 0;
    const INTERSECTION_BINDING: u32 = 1;

    pub fn new(
        device: &wgpu::Device,
        geometry_layout: &crate::RTGeometryBindGroupLayout,
        source: Option<wgpu::ShaderModuleDescriptor>,
    ) -> Self {
        let frame_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            bind_group_layouts: &[geometry_layout, &frame_bind_group_layout],
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
            frame_bind_group_layout,
            pipeline,
        }
    }

    pub fn create_frame_bind_groups(
        &self,
        device: &wgpu::Device,
        size: (u32, u32),
        out_intersections: &gpu::Buffer<uniforms::Intersection>,
        rays: &gpu::Buffer<uniforms::Ray>,
    ) -> wgpu::BindGroup {
        let pixels_count: u64 = (size.0 * size.1) as u64;
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Intersector Frame Bind Group"),
            layout: &self.frame_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: Self::RAY_BINDING,
                    resource: rays.as_sub_binding(pixels_count),
                },
                wgpu::BindGroupEntry {
                    binding: Self::INTERSECTION_BINDING,
                    resource: out_intersections.as_sub_binding(pixels_count),
                },
            ],
        })
    }

    pub fn dispatch(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        scene_bind_group: &wgpu::BindGroup,
        frame_bind_group: &wgpu::BindGroup,
        dispatch_size: (u32, u32, u32),
    ) {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Intersector Pass"),
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, scene_bind_group, &[]);
        pass.set_bind_group(1, frame_bind_group, &[]);
        pass.dispatch_workgroups(dispatch_size.0, dispatch_size.1, dispatch_size.2);
    }
}
