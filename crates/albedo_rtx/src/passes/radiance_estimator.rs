use crate::get_dispatch_size;
use crate::macros::path_separator;
use crate::uniforms;
use crate::RTGeometryBindGroupLayout;
use crate::RTSurfaceBindGroupLayout;
use albedo_backend::data::ShaderCache;
use albedo_backend::gpu;
use albedo_backend::gpu::ComputePipeline;

pub struct ShadingPass {
    frame_bind_group_layout: wgpu::BindGroupLayout,
    pipeline_layout: wgpu::PipelineLayout,
    pipeline: wgpu::ComputePipeline,
}

impl ShadingPass {
    const WORKGROUP_SIZE: (u32, u32, u32) = (8, 8, 1);

    const RAY_BINDING: u32 = 0;
    const INTERSECTION_BINDING: u32 = 1;
    const PER_DRAW_STRUCT_BINDING: u32 = 2;

    pub fn new(
        device: &wgpu::Device,
        processor: &ShaderCache,
        geometry_layout: &RTGeometryBindGroupLayout,
        surface_layout: &RTSurfaceBindGroupLayout,
    ) -> Self {
        let frame_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Radiance View Bind Group Layout"),
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
                    wgpu::BindGroupLayoutEntry {
                        binding: Self::PER_DRAW_STRUCT_BINDING,
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
            label: Some("Radiance Estimator Pipeline Layout"),
            bind_group_layouts: &[geometry_layout, surface_layout, &frame_bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = <ShadingPass as ComputePipeline>::compile(device, processor, &pipeline_layout, include_str!(concat!(
            "..",
            path_separator!(),
            "..",
            path_separator!(),
            "shaders",
            path_separator!(),
            "shading.comp"
        ))).unwrap();

        Self {
            frame_bind_group_layout,
            pipeline_layout,
            pipeline,
        }
    }

    pub fn create_frame_bind_groups(
        &self,
        device: &wgpu::Device,
        size: (u32, u32),
        out_rays: &gpu::Buffer<uniforms::Ray>,
        intersections: &gpu::Buffer<uniforms::Intersection>,
        global_uniforms: &gpu::UniformBufferSlice<uniforms::PerDrawUniforms>,
    ) -> wgpu::BindGroup {
        let pixels_count: u64 = (size.0 * size.1) as u64;
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Radiance Estimator Frame Bind Group"),
            layout: &self.frame_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: Self::RAY_BINDING,
                    resource: out_rays.as_sub_binding(pixels_count),
                },
                wgpu::BindGroupEntry {
                    binding: Self::INTERSECTION_BINDING,
                    resource: intersections.as_sub_binding(pixels_count),
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
        geometry_bindgroup: &wgpu::BindGroup,
        surface_bindgroup: &wgpu::BindGroup,
        frame_bind_groups: &wgpu::BindGroup,
        size: (u32, u32, u32),
    ) {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Shading Pass"),
            timestamp_writes: None,
        });
        let workgroups = get_dispatch_size(&size, &Self::WORKGROUP_SIZE);
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, geometry_bindgroup, &[]);
        pass.set_bind_group(1, surface_bindgroup, &[]);
        pass.set_bind_group(2, frame_bind_groups, &[]);
        pass.dispatch_workgroups(workgroups.0, workgroups.1, workgroups.2);
    }
}

impl ComputePipeline for ShadingPass {
    const LABEL: &'static str = "Radiance Estimator Pipeline";
    const SHADER_ID: &'static str = "shading.comp";

    fn set_pipeline(&mut self, pipeline: wgpu::ComputePipeline) {
        self.pipeline = pipeline;
    }
    fn get_pipeline_layout(&self) -> &wgpu::PipelineLayout {
        &self.pipeline_layout
    }
}
