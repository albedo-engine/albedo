use crate::renderer::{UniformsGPU, resources};
use albedo_backend::{GPUBuffer, UniformBuffer, shader_bindings};

pub struct GPURadianceEstimator {
    bind_group_layouts: [wgpu::BindGroupLayout; 2],
    pipeline: wgpu::ComputePipeline,
    base_bind_group: Option<wgpu::BindGroup>,
    // targets_bind_group: Option<[wgpu::BindGroup; 2]>,
    target_bind_group: Option<wgpu::BindGroup>,
}

impl GPURadianceEstimator {
    pub fn new(device: &wgpu::Device) -> Self {
        let bind_group_layouts = [
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Radiance Estimator Base Layout"),
                entries: &[
                    shader_bindings::buffer_entry(0, wgpu::ShaderStage::COMPUTE, false),
                    shader_bindings::buffer_entry(1, wgpu::ShaderStage::COMPUTE, true),
                    shader_bindings::buffer_entry(2, wgpu::ShaderStage::COMPUTE, true),
                    shader_bindings::buffer_entry(3, wgpu::ShaderStage::COMPUTE, true),
                    shader_bindings::buffer_entry(4, wgpu::ShaderStage::COMPUTE, true),
                    shader_bindings::buffer_entry(5, wgpu::ShaderStage::COMPUTE, true),
                    shader_bindings::buffer_entry(6, wgpu::ShaderStage::COMPUTE, true),
                    shader_bindings::uniform_entry(7, wgpu::ShaderStage::COMPUTE),
                    shader_bindings::sampler_entry(8, wgpu::ShaderStage::COMPUTE, true),
                    shader_bindings::texture2d_entry(9, wgpu::ShaderStage::COMPUTE),
                ],
            }),
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Radiance Estimator Render Target Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::COMPUTE,
                        ty: shader_bindings::storage_texture2d(
                            wgpu::TextureFormat::Rgba32Float,
                            wgpu::StorageTextureAccess::WriteOnly,
                        ),
                        count: None,
                    },
                    shader_bindings::uniform_entry(1, wgpu::ShaderStage::COMPUTE),
                ],
            }),
        ];

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Radiance Estimator Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layouts[0], &bind_group_layouts[1]],
            push_constant_ranges: &[],
        });

        let shader =
            device.create_shader_module(&wgpu::include_spirv!("../shaders/shading.comp.spv"));

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Radiance Estimator Pipeline"),
            layout: Some(&pipeline_layout),
            entry_point: "main",
            module: &shader,
        });

        GPURadianceEstimator {
            bind_group_layouts,
            pipeline,
            base_bind_group: None,
            target_bind_group: None,
        }
    }

    pub fn bind_buffers(
        &mut self,
        device: &wgpu::Device,
        out_rays: &GPUBuffer<resources::RayGPU>,
        intersections: &GPUBuffer<resources::IntersectionGPU>,
        instances: &GPUBuffer<resources::InstanceGPU>,
        indices: &GPUBuffer<u32>,
        vertices: &GPUBuffer<resources::VertexGPU>,
        lights: &GPUBuffer<resources::LightGPU>,
        materials: &GPUBuffer<resources::MaterialGPU>,
        scene_info: &UniformBuffer<resources::SceneSettingsGPU>,
        probe_view: &wgpu::TextureView,
        probe_sampler: &wgpu::Sampler
    ) {
        self.base_bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Radiance Estimator Base Bind Group"),
            layout: &self.bind_group_layouts[0],
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: out_rays.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: intersections.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: instances.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: indices.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: vertices.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: lights.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: materials.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: scene_info.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: wgpu::BindingResource::Sampler(probe_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: wgpu::BindingResource::TextureView(probe_view),
                },
            ],
        }));
    }

    pub fn bind_target(
        &mut self,
        device: &wgpu::Device,
        view: &wgpu::TextureView,
        global_uniforms: &UniformBuffer<resources::GlobalUniformsGPU>
    ) {
        self.target_bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Radiance Esimator Render Target Bind Group"),
            layout: &self.bind_group_layouts[1],
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: global_uniforms.as_entire_binding(),
                }
            ],
        }));
    }

    pub fn run(&self, encoder: &mut wgpu::CommandEncoder, width: u32, height: u32) {
        match (&self.base_bind_group, &self.target_bind_group) {
            (Some(base_group), Some(target_group)) => {
                let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Radiance Estimator Compute Pass"),
                });
                compute_pass.set_pipeline(&self.pipeline);
                compute_pass.set_bind_group(0, base_group, &[]);
                compute_pass.set_bind_group(1, target_group, &[]);
                // @todo: how to deal with hardcoded size.
                compute_pass.dispatch(width / 8, height / 8, 1);
            }
            _ => (),
        }
    }
}
