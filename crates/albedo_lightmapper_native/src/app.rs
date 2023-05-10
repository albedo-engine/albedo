use albedo_backend::gpu;
use albedo_rtx::uniforms;
use futures;

pub struct GpuContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
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

        Self { device, queue }
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}

/// GPU data for a single mesh.
pub struct MeshData {
    index_buffer: gpu::IndexBuffer,
    vertex_buffer: gpu::Buffer<uniforms::Vertex>,
}

impl MeshData {
    pub fn new(device: &wgpu::Device, vertices: &[uniforms::Vertex], indices: &[u32]) -> Self {
        Self {
            vertex_buffer: gpu::Buffer::new_vertex_with_data(device, vertices, None),
            index_buffer: gpu::IndexBuffer::new_with_data_32(device, indices, None),
        }
    }
    pub fn vertices(&self) -> &gpu::Buffer<uniforms::Vertex> {
        &self.vertex_buffer
    }
    pub fn indices(&self) -> &gpu::IndexBuffer {
        &self.index_buffer
    }
}

pub struct App {
    pub context: GpuContext,
    pub data: Option<MeshData>,
}

impl App {
    pub fn new() -> Self {
        App {
            context: GpuContext::new(),
            data: None,
        }
    }
}
