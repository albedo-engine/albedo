use std::{borrow::Cow, result};
use accel::BVHNodeGPU;
use glam;
use wgpu::{BindGroup, BindGroupLayout, BindGroupLayoutEntry, BindingType, CommandEncoder, ComputePipeline, ComputePipelineDescriptor, Device, PipelineLayoutDescriptor, ShaderStage, StorageTextureAccess, TextureFormat, TextureViewDimension};

use albedo_backend::{GPUBuffer, UniformBuffer, shader_bindings};
use crate::accel;

#[repr(C)]
#[derive(Clone, Copy)]
struct MaterialGPU {
    color: glam::Vec4,
}
unsafe impl bytemuck::Pod for MaterialGPU {}
unsafe impl bytemuck::Zeroable for MaterialGPU {}

#[repr(C)]
#[derive(Clone, Copy)]
struct VertexGPU {
  position: glam::Vec3,
  padding_0: u32,
  norma: glam::Vec3,
  padding_1: u32,
  // @todo: add UV
}
unsafe impl bytemuck::Pod for VertexGPU {}
unsafe impl bytemuck::Zeroable for VertexGPU {}

#[repr(C)]
#[derive(Clone, Copy)]
struct LightGPU {
    normal: glam::Vec4,
    tangent: glam::Vec4,
    bitangent: glam::Vec4,
    intensity: f32,
    padding_0: u32,
    padding_1: u32,
    padding_2: u32,
}
unsafe impl bytemuck::Pod for LightGPU {}
unsafe impl bytemuck::Zeroable for LightGPU {}

#[repr(C)]
#[derive(Clone, Copy)]
struct UniformsGPU {
    time: f32,
}
unsafe impl bytemuck::Pod for UniformsGPU {}
unsafe impl bytemuck::Zeroable for UniformsGPU {}

#[repr(C)]
#[derive(Clone, Copy)]
struct CameraGPU {
    origin: glam::Vec3,
    v_fov: f32,
    up: glam::Vec3,
    padding_0: f32,
    right: glam::Vec3,
    padding_1: f32,
}
unsafe impl bytemuck::Pod for CameraGPU {}
unsafe impl bytemuck::Zeroable for CameraGPU {}

#[repr(C)]
#[derive(Clone, Copy)]
struct RenderInfo {
    width: u32,
    height: u32,
    instanceCount: u32,
    lightCount: u32,
    frameCount: u32,
}
unsafe impl bytemuck::Pod for RenderInfo {}
unsafe impl bytemuck::Zeroable for RenderInfo {}

#[repr(C)]
#[derive(Clone, Copy)]
struct Instance {
    world_to_model: glam::Mat4,
    bvh_root_index: u32,
    material_index: u32,
    padding_0: u32,
    padding_1: u32,
}
unsafe impl bytemuck::Pod for Instance {}
unsafe impl bytemuck::Zeroable for Instance {}

struct PathtracePassResources {
    pub(crate) render_info: UniformBuffer<RenderInfo>,
    pub(crate) instances: GPUBuffer<Instance>,
    pub(crate) nodes: GPUBuffer<accel::BVHNodeGPU>,
    pub(crate) index_buffer: GPUBuffer<u32>,
    pub(crate) vertex_buffer: GPUBuffer<VertexGPU>,
    pub(crate) material_buffer: GPUBuffer<MaterialGPU>,
    pub(crate) light_buffer: GPUBuffer<LightGPU>,
    pub(crate) uniform_buffer: UniformBuffer<UniformsGPU>,
    pub(crate) camera_uniform_buffer: UniformBuffer<CameraGPU>,
    pub render_target: wgpu::Texture,
    pub render_target_view: wgpu::TextureView,
}

struct PathtracePass {
    gpu_resources: PathtracePassResources,
    bind_group_layouts: [wgpu::BindGroupLayout; 3],
    targets_bind_group: BindGroup,
    scene_bind_group: BindGroup,
    camera_bind_group: BindGroup,
    pipeline: ComputePipeline
}

impl PathtracePass {

    pub fn new(device: &Device) -> PathtracePass {
        let bind_group_layouts = [
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    // BindGroupLayoutEntry {
                    //     binding: 0,
                    //     visibility: ShaderStage::COMPUTE,
                    //     ty: shader_bindings::storage_texture2d(wgpu::TextureFormat::Rgba32Float, wgpu::StorageTextureAccess::ReadOnly),
                    //     count: None,
                    // },
                    // BindGroupLayoutEntry {
                    //     binding: 1,
                    //     visibility: ShaderStage::COMPUTE,
                    //     ty: shader_bindings::storage_texture2d(wgpu::TextureFormat::Rgba32Float, wgpu::StorageTextureAccess::WriteOnly),
                    //     count: None,
                    // },
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStage::COMPUTE,
                        ty: shader_bindings::storage_texture2d(wgpu::TextureFormat::Rgba32Float, wgpu::StorageTextureAccess::WriteOnly),
                        count: None,
                    },
                ],
            }),
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStage::COMPUTE,
                        ty: shader_bindings::uniform(),
                        count: None
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStage::COMPUTE,
                        ty: shader_bindings::buffer(true),
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStage::COMPUTE,
                        ty: shader_bindings::buffer(true),
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 3,
                        visibility: ShaderStage::COMPUTE,
                        ty: shader_bindings::buffer(true),
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 4,
                        visibility: ShaderStage::COMPUTE,
                        ty: shader_bindings::buffer(true),
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 5,
                        visibility: ShaderStage::COMPUTE,
                        ty: shader_bindings::buffer(true),
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 6,
                        visibility: ShaderStage::COMPUTE,
                        ty: shader_bindings::buffer(true),
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 7,
                        visibility: ShaderStage::COMPUTE,
                        ty: shader_bindings::uniform(),
                        count: None
                    },
                    // BindGroupLayoutEntry {
                    //     binding: 8,
                    //     visibility: ShaderStage::COMPUTE,
                    //     ty: wgpu::BindingType::Texture {
                    //         multisampled: false,
                    //         sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    //         view_dimension: wgpu::TextureViewDimension::D2,
                    //     },
                    //     count: None
                    // },
                    // BindGroupLayoutEntry {
                    //     binding: 11,
                    //     visibility: ShaderStage::COMPUTE,
                    //     ty: wgpu::BindingType::Sampler {
                    //         comparison: false,
                    //         filtering: true,
                    //     },
                    //     count: None
                    // }
                ],
            }),
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStage::COMPUTE,
                        ty: shader_bindings::uniform(),
                        count: None
                    }
                ]
            })
        ];

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Pathtrace Pipeline"),
            bind_group_layouts: &[
                &bind_group_layouts[0],
                &bind_group_layouts[1],
                &bind_group_layouts[2],
            ],
            push_constant_ranges: &[]
        });

        // @todo: move shade compilation out.
        let cs_module = device.create_shader_module(&wgpu::include_spirv!("../shaders/pathtrace.comp.spv"));

        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Pahtracing Pipeline"),
            layout: Some(&pipeline_layout),
            entry_point: "main",
            module: &cs_module
        });

        let render_target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Pathtrace Render Target"),
            size: wgpu::Extent3d { width: 640, height: 480, depth: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::STORAGE
        });
        let render_target_view = render_target.create_view(&wgpu::TextureViewDescriptor::default());

        let gpu_resources = PathtracePassResources {
            render_info: UniformBuffer::new(device),
            instances: GPUBuffer::new_with_count(device, 1),
            nodes: GPUBuffer::new_with_count(device, 1),
            index_buffer: GPUBuffer::new_with_count(device, 1),
            vertex_buffer: GPUBuffer::new_with_count(device, 1),
            material_buffer: GPUBuffer::new_with_count(device, 1),
            light_buffer: GPUBuffer::new_with_count(device, 1),
            uniform_buffer: UniformBuffer::new(device),
            camera_uniform_buffer: UniformBuffer::new(device),
            render_target,
            render_target_view
        };

        let targets_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Pathtracing Render Targets"),
            layout: &bind_group_layouts[0],
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&gpu_resources.render_target_view)
                },
                // wgpu::BindGroupEntry {
                //     binding: 1, // @todo: add write target here.
                //     resource: wgpu::BindingResource::TextureView(&gpu_resources.render_target_view)
                // },
            ]
        });

        let scene_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Pathtracing Scene"),
            layout: &bind_group_layouts[1],
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: gpu_resources.render_info.as_entire_binding()
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: gpu_resources.instances.as_entire_binding()
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: gpu_resources.nodes.as_entire_binding()
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: gpu_resources.index_buffer.as_entire_binding()
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: gpu_resources.vertex_buffer.as_entire_binding()
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: gpu_resources.material_buffer.as_entire_binding()
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: gpu_resources.light_buffer.as_entire_binding()
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: gpu_resources.uniform_buffer.as_entire_binding()
                },
                // @todo: add probes
                // @todo: add materials
                // @todo: add textures
            ]
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Pathtracing Camera"),
            layout: &bind_group_layouts[2],
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: gpu_resources.camera_uniform_buffer.as_entire_binding()
                }
            ]
        });

        PathtracePass {
            bind_group_layouts,
            scene_bind_group,
            camera_bind_group,
            targets_bind_group,
            pipeline,
            gpu_resources
        }
    }

    pub fn render(&self, frame: &wgpu::SwapChainTexture, queue: &wgpu::Queue, encoder: &mut CommandEncoder) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None
        });
        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_bind_group(0, &self.targets_bind_group, &[]);
        compute_pass.set_bind_group(1, &self.scene_bind_group, &[]);
        compute_pass.set_bind_group(2, &self.camera_bind_group, &[]);
    }

}

pub struct Renderer {
    pathtrace_pass: PathtracePass
}

// @todo: split renderer and intersector.
// This may slightly reduce performance but then the intersector is reusable.
impl Renderer {

    pub fn new (device: &Device) -> Renderer {
        Renderer {
            pathtrace_pass: PathtracePass::new(device)
        }
    }

    pub fn render(&self, device: &wgpu::Device, frame: &wgpu::SwapChainTexture, queue: &wgpu::Queue) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        self.pathtrace_pass.render(frame, queue, &mut encoder);
    }


}
