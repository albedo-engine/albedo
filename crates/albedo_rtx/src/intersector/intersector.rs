pub struct GPUIntersector {
    bind_group: wgpu::BindGroup,
}

impl GPUIntersector {

    pub fn new(
        device: &wgpu::Device,
        nodes: &wgpu::Buffer,
        indices: &wgpu::Buffer,
        vertices: &wgpu::Buffer,
        lights: &wgpu::Buffer,
        rays: &wgpu::Buffer,
        scene_info: &wgpu::Buffer
    ) -> GPUIntersector {

        GPUIntersector {

        }
    }

}
