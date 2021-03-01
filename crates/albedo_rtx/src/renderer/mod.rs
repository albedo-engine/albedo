mod blit_pass;
use blit_pass::BlitPass;

pub mod resources;
pub mod utils;

use accel::BVHNodeGPU;
use glam;
use wgpu::{
    BindGroup, BindGroupLayoutEntry, CommandEncoder, ComputePipeline, ComputePipelineDescriptor,
    Device, PipelineLayoutDescriptor, ShaderStage,
};

use crate::accel;
use albedo_backend::{shader_bindings, GPUBuffer, UniformBuffer};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct UniformsGPU {
    time: f32,
}
unsafe impl bytemuck::Pod for UniformsGPU {}
unsafe impl bytemuck::Zeroable for UniformsGPU {}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct CameraGPU {
    pub origin: glam::Vec3,
    pub v_fov: f32,
    pub up: glam::Vec3,
    padding_0: f32,
    pub right: glam::Vec3,
    padding_1: f32,
}

impl CameraGPU {

    pub fn new() -> Self {
        CameraGPU {
            origin: glam::Vec3::new(0.0, 0.0, 2.0),
            v_fov: 0.78,
            up: glam::Vec3::new(0.0, 1.0, 0.0),
            right: glam::Vec3::new(1.0, 0.0, 0.0),
            ..Default::default()
        }
    }

}

unsafe impl bytemuck::Pod for CameraGPU {}
unsafe impl bytemuck::Zeroable for CameraGPU {}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct RenderInfo {
    width: u32,
    height: u32,
    instance_count: u32,
    light_count: u32,
    frame_count: u32,
    padding_0: f32,
    padding_1: f32,
    padding_2: f32,
}

impl RenderInfo {

    pub fn new(width: u32, height: u32) -> Self {
        RenderInfo {
            width, height,
            ..Default::default()
        }
    }

}

unsafe impl bytemuck::Pod for RenderInfo {}
unsafe impl bytemuck::Zeroable for RenderInfo {}

struct PathtracePassResources {
    pub(crate) render_info: UniformBuffer<RenderInfo>,
    pub(crate) instances: GPUBuffer<resources::InstanceGPU>,
    pub(crate) nodes: GPUBuffer<accel::BVHNodeGPU>,
    pub(crate) index_buffer: GPUBuffer<u32>,
    pub(crate) vertex_buffer: GPUBuffer<resources::VertexGPU>,
    pub(crate) material_buffer: GPUBuffer<resources::MaterialGPU>,
    pub(crate) light_buffer: GPUBuffer<resources::LightGPU>,
    pub(crate) uniform_buffer: UniformBuffer<UniformsGPU>,
    pub(crate) camera_uniform_buffer: UniformBuffer<CameraGPU>,
    pub render_target: wgpu::Texture,
    pub render_target_view: wgpu::TextureView,
    pub render_target_sampler: wgpu::Sampler,
}

struct PathtracePass {
    gpu_resources: PathtracePassResources,
    bind_group_layouts: [wgpu::BindGroupLayout; 3],
    targets_bind_group: BindGroup,
    scene_bind_group: BindGroup,
    camera_bind_group: BindGroup,
    pipeline: ComputePipeline,
    render_info: RenderInfo,
    needs_bind_group_update: bool,
    needs_render_info_update: bool,
}

impl PathtracePass {
    pub fn new(device: &Device, width: u32, height: u32) -> PathtracePass {
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
                        ty: shader_bindings::storage_texture2d(
                            wgpu::TextureFormat::Rgba32Float,
                            wgpu::StorageTextureAccess::WriteOnly,
                        ),
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
                        count: None,
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
                        count: None,
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
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::COMPUTE,
                    ty: shader_bindings::uniform(),
                    count: None,
                }],
            }),
        ];

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Pathtrace Pipeline"),
            bind_group_layouts: &[
                &bind_group_layouts[0],
                &bind_group_layouts[1],
                &bind_group_layouts[2],
            ],
            push_constant_ranges: &[],
        });

        // @todo: move shade compilation out.
        let cs_module =
            device.create_shader_module(&wgpu::include_spirv!("../shaders/pathtrace.comp.spv"));

        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Pahtracing Pipeline"),
            layout: Some(&pipeline_layout),
            entry_point: "main",
            module: &cs_module,
        });

        let render_target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Pathtrace Render Target"),
            size: wgpu::Extent3d {
                width: 640,
                height: 480,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::STORAGE,
        });
        let render_target_view = render_target.create_view(&wgpu::TextureViewDescriptor::default());
        let render_target_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

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
            render_target_view,
            render_target_sampler,
        };

        let targets_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Pathtracing Render Targets"),
            layout: &bind_group_layouts[0],
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&gpu_resources.render_target_view),
                },
                // wgpu::BindGroupEntry {
                //     binding: 1, // @todo: add write target here.
                //     resource: wgpu::BindingResource::TextureView(&gpu_resources.render_target_view)
                // },
            ],
        });

        let scene_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Pathtracing Scene"),
            layout: &bind_group_layouts[1],
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: gpu_resources.render_info.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: gpu_resources.instances.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: gpu_resources.nodes.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: gpu_resources.index_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: gpu_resources.vertex_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: gpu_resources.material_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: gpu_resources.light_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: gpu_resources.uniform_buffer.as_entire_binding(),
                },
                // @todo: add probes
                // @todo: add materials
                // @todo: add textures
            ],
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Pathtracing Camera"),
            layout: &bind_group_layouts[2],
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: gpu_resources.camera_uniform_buffer.as_entire_binding(),
            }],
        });

        PathtracePass {
            bind_group_layouts,
            scene_bind_group,
            camera_bind_group,
            targets_bind_group,
            pipeline,
            gpu_resources,
            needs_bind_group_update: false,
            needs_render_info_update: true,
            render_info: RenderInfo::new(width, height)
        }
    }

    pub fn update_resources(&mut self, device: &wgpu::Device) {
        self.scene_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Pathtracing Scene"),
            layout: &self.bind_group_layouts[1],
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.gpu_resources.render_info.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.gpu_resources.instances.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.gpu_resources.nodes.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.gpu_resources.index_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: self.gpu_resources.vertex_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: self.gpu_resources.material_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: self.gpu_resources.light_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: self.gpu_resources.uniform_buffer.as_entire_binding(),
                },
                // @todo: add probes
                // @todo: add materials
                // @todo: add textures
            ],
        });
    }

    pub fn render(
        &mut self,
        device: &Device,
        frame: &wgpu::SwapChainTexture,
        queue: &wgpu::Queue,
        encoder: &mut CommandEncoder,
    ) {
        if self.needs_bind_group_update {
            self.update_resources(device);
            self.needs_bind_group_update = false;
        }
        if self.needs_render_info_update {
            self.gpu_resources.render_info.update(queue, &self.render_info);
            self.needs_render_info_update = false;
        }
        // @todo: do not harcode.
        let RenderInfo { width, height, .. } = self.render_info;
        let mut compute_pass =
            encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_bind_group(0, &self.targets_bind_group, &[]);
        compute_pass.set_bind_group(1, &self.scene_bind_group, &[]);
        compute_pass.set_bind_group(2, &self.camera_bind_group, &[]);
        compute_pass.dispatch(width / 8, height / 8, 1)
    }
}

pub struct Renderer {
    pathtrace_pass: PathtracePass,
    blit_pass: BlitPass,
}

// @todo: split renderer and intersector.
// This may slightly reduce performance but then the intersector is reusable.
impl Renderer {
    pub fn new(device: &Device, swap_chain_format: wgpu::TextureFormat) -> Renderer {
        // @todo: do not hardcode
        let width: u32 = 640;
        let height: u32 = 480;

        let pathtrace_pass = PathtracePass::new(device, width, height);
        let mut blit_pass = BlitPass::new(device, swap_chain_format);
        // @todo: refactor init somewhere.
        blit_pass.init(
            device,
            &pathtrace_pass.gpu_resources.render_target_view,
            &pathtrace_pass.gpu_resources.render_target_sampler,
            pathtrace_pass.gpu_resources.render_info.as_entire_binding(),
        );
        Renderer {
            pathtrace_pass,
            blit_pass,
        }
    }

    // @todo: those `commit` methods are just to try out the entire pipeline.
    // Remove them.

    pub fn commit_camera(&mut self, queue: &wgpu::Queue, camera: &CameraGPU) {
        self.pathtrace_pass.gpu_resources.camera_uniform_buffer.update(queue, camera);
    }

    pub fn commit_instances(&mut self, instances: &[resources::InstanceGPU], device: &Device, queue: &wgpu::Queue) {
        // @todo: authorize offset. Should I just expose the gpu resources
        // to the user and he does everything?
        if !self.pathtrace_pass.gpu_resources.instances.fits(instances) {
            self.pathtrace_pass.gpu_resources.instances = GPUBuffer::from_data(device, instances);
            // @todo: better if the access to the buffer was controlled and so
            // this flag would be automatically handled
            self.pathtrace_pass.needs_bind_group_update = true;
        }
        self.pathtrace_pass.render_info.instance_count = instances.len() as u32;
        self.pathtrace_pass.needs_render_info_update = true;
        self.pathtrace_pass.gpu_resources.instances.update(queue, instances);
    }

    pub fn commit_lights(&mut self, lights: &[resources::LightGPU], device: &Device, queue: &wgpu::Queue) {
        // @todo: authorize offset. Should I just expose the gpu resources
        // to the user and he does everything?
        if !self.pathtrace_pass.gpu_resources.light_buffer.fits(lights) {
            self.pathtrace_pass.gpu_resources.light_buffer = GPUBuffer::from_data(device, lights);

            // @todo: better if the access to the buffer was controlled and so
            // this flag would be automatically handled
            self.pathtrace_pass.needs_bind_group_update = true;
        }
        self.pathtrace_pass.render_info.light_count = lights.len() as u32;
        self.pathtrace_pass.needs_render_info_update = true;
        self.pathtrace_pass.gpu_resources.light_buffer.update(queue, lights);
    }

    pub fn commit_bvh(&mut self, bvhs: &[BVHNodeGPU], device: &Device, queue: &wgpu::Queue) {
        // @todo: authorize offset. Should I just expose the gpu resources
        // to the user and he does everything?
        if !self.pathtrace_pass.gpu_resources.nodes.fits(bvhs) {
            self.pathtrace_pass.gpu_resources.nodes = GPUBuffer::from_data(device, bvhs);
            // @todo: better if the access to the buffer was controlled and so
            // this flag would be automatically handled
            self.pathtrace_pass.needs_bind_group_update = true;

        }
        self.pathtrace_pass.gpu_resources.nodes.update(queue, bvhs);
    }

    pub fn commit_vertices(&mut self, vertices: &[resources::VertexGPU], device: &Device, queue: &wgpu::Queue) {
        // @todo: authorize offset. Should I just expose the gpu resources
        // to the user and he does everything?
        if !self.pathtrace_pass.gpu_resources.vertex_buffer.fits(vertices) {
            self.pathtrace_pass.gpu_resources.vertex_buffer = GPUBuffer::from_data(device, vertices);
            self.pathtrace_pass.needs_bind_group_update = true;
        }
        self.pathtrace_pass.gpu_resources.vertex_buffer.update(queue, vertices);
    }

    pub fn commit_indices(&mut self, indices: &[u32], device: &Device, queue: &wgpu::Queue) {
        // @todo: authorize offset. Should I just expose the gpu resources
        // to the user and he does everything?
        if !self.pathtrace_pass.gpu_resources.index_buffer.fits(indices) {
            self.pathtrace_pass.gpu_resources.index_buffer = GPUBuffer::from_data(device, indices);
            self.pathtrace_pass.needs_bind_group_update = true;
        }
        self.pathtrace_pass.gpu_resources.index_buffer.update(queue, indices);
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        frame: &wgpu::SwapChainTexture,
        queue: &wgpu::Queue,
    ) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        self.pathtrace_pass.render(device, frame, queue, &mut encoder);
        self.blit_pass.render(frame, queue, &mut encoder);
        queue.submit(Some(encoder.finish()));
    }
}
