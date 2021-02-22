use std::borrow::Cow;
use accel::BVHNodeGPU;
use glam;
use wgpu::{BindGroup, BindGroupLayout, BindGroupLayoutEntry, BindingType, BufferBindingType, CommandEncoder, ComputePipeline, ComputePipelineDescriptor, Device, PipelineLayoutDescriptor, ShaderStage, StorageTextureAccess, TextureFormat, TextureViewDimension};

use albedo_backend::{GPUBuffer};
use crate::accel;

struct RenderInfo {
    width: u32,
    height: u32,
    instanceCount: u32,
    lightCount: u32,
    frameCount: u32,
}

struct Instance
{
    world_to_model: glam::Mat4,
    bvh_root_index: u32,
    material_index: u32,
    padding_0: u32,
    padding_1: u32,
}

struct PathtracePassResources {
    render_info: GPUBuffer<RenderInfo>,
    instances: GPUBuffer<Instance>,
    nodes: GPUBuffer<accel::BVHNodeGPU>,
    indicesBuffer: GPUBuffer<u32>,
    vertexBuffer: GPUBuffer<todo>,
    materialBuffer: GPUBuffer<todo>,
    lightsBuffer: GPUBuffer<todo>,
    uniformsBuffer: GPUBuffer<todo>,
    ew: GPUBuffer<todo>,
    ew2: GPUBuffer<todo>,
    obeView: GPUBuffer<todo>,
    probeSampler: GPUBuffer<todo>
}

struct PathtracePass<'a> {
    device: &'a Device,
    bindGroup: BindGroup,
    pipeline: ComputePipeline
}

impl<'a> PathtracePass<'a> {

    pub fn new(device: &Device) -> PathtracePass {
        // @todo: refactor and generate entry using something simpler.
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    count: None
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage {
                            read_only: true
                        },
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStage::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage {
                            read_only: true
                        },
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStage::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage {
                            read_only: true
                        },
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStage::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage {
                            read_only: true
                        },
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 5,
                    visibility: ShaderStage::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage {
                            read_only: true
                        },
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 6,
                    visibility: ShaderStage::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage {
                            read_only: true
                        },
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 7,
                    visibility: ShaderStage::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    count: None
                },
                BindGroupLayoutEntry {
                    binding: 8,
                    visibility: ShaderStage::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadOnly,
                        view_dimension: TextureViewDimension::D2,
                        format: TextureFormat::R32Float
                    },
                    count: None
                },
                BindGroupLayoutEntry {
                    binding: 9,
                    visibility: ShaderStage::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        view_dimension: TextureViewDimension::D2,
                        format: TextureFormat::R32Float
                    },
                    count: None
                },
                BindGroupLayoutEntry {
                    binding: 10,
                    visibility: ShaderStage::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None
                },
                BindGroupLayoutEntry {
                    binding: 11,
                    visibility: ShaderStage::COMPUTE,
                    ty: wgpu::BindingType::Sampler {
                        comparison: false,
                        filtering: true,
                    },
                    count: None
                }
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
            label: None,
            push_constant_ranges: &[]
        });

        // @todo: move shade compilation out.
        let cs_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::SpirV(Cow::Borrowed(include_str!("../shaders/pathtrace.comp.spv"))),
            flags: wgpu::ShaderFlags::VALIDATION,
        });

        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("pathtracing-pipeline"),
            layout: Some(&pipeline_layout),
            entry_point: "main",
            module: &cs_module
        });

        PathtracePass {
            device,
            pipeline
        }
    }

    pub fn render(&self, frame: &wgpu::SwapChainTexture, queue: &wgpu::Queue, encoder: &mut CommandEncoder) {
        let compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None
        });
        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_bind_group(0, &self.bind_group, &[]);
    }

}

pub struct Renderer<'a> {
    device: &'a Device,
    pathtrace_pass: PathtracePass<'a>
}

// @todo: split renderer and intersector.
// This may slightly reduce performance but then the intersector is reusable.
impl<'a> Renderer<'a> {

    pub fn new (device: &'a Device) -> Renderer {
        Renderer {
            device,
            pathtrace_pass: PathtracePass::new(device)
        }
    }

    pub fn render(&self, frame: &wgpu::SwapChainTexture, queue: &wgpu::Queue) {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        self.pathtrace_pass.render(frame, queue, &mut encoder);
    }


}
