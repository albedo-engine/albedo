use albedo_backend::{shader_bindings};

pub struct GPUIntersector {
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl GPUIntersector {

    pub fn new(device: &wgpu::Device) -> GPUIntersector {

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("GPUIntersector Layout"),
            entries: &[
                shader_bindings::buffer_entry(binding: 0, wgpu::ShaderStage::COMPUTE, true),
                shader_bindings::buffer_entry(binding: 1, wgpu::ShaderStage::COMPUTE, true),
                shader_bindings::buffer_entry(binding: 2, wgpu::ShaderStage::COMPUTE, true),
                shader_bindings::buffer_entry(binding: 3, wgpu::ShaderStage::COMPUTE, true),
                shader_bindings::buffer_entry(binding: 4, wgpu::ShaderStage::COMPUTE, true),
                shader_bindings::buffer_entry(binding: 5, wgpu::ShaderStage::COMPUTE, true),
                shader_bindings::buffer_entry(binding: 6, wgpu::ShaderStage::COMPUTE, false),
                shader_bindings::uniform_entry(binding: 7, wgpu::ShaderStage::COMPUTE),
            ],
        });

        GPUIntersector {
            bind_group_layout
        }
    }

    pub fn set_buffers(
        nodes: &wgpu::Buffer,
        indices: &wgpu::Buffer,
        vertices: &wgpu::Buffer,
        lights: &wgpu::Buffer,
        rays: &wgpu::Buffer,
        scene_info: &wgpu::Buffer
    ) {

    }

}
