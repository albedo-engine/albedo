pub trait ResourceBuilder {
    type Resource;
    fn build(self, device: &wgpu::Device) -> Result<Self::Resource, String>;
}
