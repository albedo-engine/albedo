use albedo_backend::gpu;
use albedo_rtx::uniforms::{Instance, Vertex};
use futures;
// use renderdoc::{RenderDoc, V141};

pub struct GpuContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub sampler_nearest: wgpu::Sampler,
    pub sampler_linear: wgpu::Sampler,
    // pub renderdoc: Option<RenderDoc<V141>>,
}

impl GpuContext {
    pub fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            dx12_shader_compiler: wgpu::util::dx12_shader_compiler_from_env().unwrap_or_default(),
        });
        let adapter =
            futures::executor::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            }))
            .expect("No suitable GPU adapters found on the system!");

        let optional_features: wgpu::Features = wgpu::Features::default();
        let required_features: wgpu::Features =
            wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES;

        let adapter_features: wgpu::Features = wgpu::Features::default();
        let needed_limits = wgpu::Limits {
            max_storage_buffers_per_shader_stage: 8,
            max_storage_buffer_binding_size: 256 * 1024 * 1024,
            ..wgpu::Limits::default()
        };
        let trace_dir = std::env::var("WGPU_TRACE");

        let (device, queue) = futures::executor::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: (optional_features & adapter_features) | required_features,
                limits: needed_limits,
            },
            trace_dir.ok().as_ref().map(std::path::Path::new),
        ))
        .expect("Unable to find a suitable GPU adapter!");

        let sampler_nearest = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let sampler_linear = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // if false {
        //     unsafe { libloading::os::windows::Library::new( "renderdoc.dll"); }
        //     let renderdoc = Some(RenderDoc::<V141>::new().unwrap());
        // }

        // let renderdoc = None;

        Self {
            device,
            queue,
            sampler_nearest,
            sampler_linear,
            // renderdoc,
        }
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}

/// GPU data for scene
pub struct SceneGPU {
    pub instance_buffer: gpu::Buffer<Instance>,
    pub bvh_buffer: gpu::Buffer<FlatNode>,
    pub index_buffer: gpu::Buffer<u32>,
    pub vertex_buffer: gpu::Buffer<Vertex>,
}

impl SceneGPU {
    pub fn new(
        device: &wgpu::Device,
        instances: &[Instance],
        bvh: &[FlatNode],
        indices: &[u32],
        vertices: &[Vertex],
    ) -> Self {
        SceneGPU {
            instance_buffer: gpu::Buffer::new_storage_with_data(&device, instances, None),
            bvh_buffer: gpu::Buffer::new_storage_with_data(&device, bvh, None),
            index_buffer: gpu::Buffer::new_storage_with_data(
                &device,
                indices,
                Some(gpu::BufferInitDescriptor {
                    label: None,
                    usage: wgpu::BufferUsages::INDEX,
                }),
            ),
            vertex_buffer: gpu::Buffer::new_storage_with_data(
                &device,
                vertices,
                Some(gpu::BufferInitDescriptor {
                    label: None,
                    usage: wgpu::BufferUsages::VERTEX,
                }),
            ),
        }
    }
}

pub struct App {
    pub context: GpuContext,
    pub scene: Option<SceneGPU>,
}

impl App {
    pub fn new() -> Self {
        App {
            context: GpuContext::new(),
            scene: None,
        }
    }
}
