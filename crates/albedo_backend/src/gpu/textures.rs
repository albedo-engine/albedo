pub struct TextureBuilder<'a> {
    desc: wgpu::TextureDescriptor<'a>,
}

impl<'a> TextureBuilder<'a> {
    pub fn new_2d(width: u32, height: u32) -> Self {
        Self {
            desc: wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: width,
                    height: height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
        }
    }

    pub fn label(mut self, label: wgpu::Label<'a>) -> Self {
        self.desc.label = label;
        self
    }

    pub fn mip_level_count(mut self, count: u32) -> Self {
        self.desc.mip_level_count = count;
        self
    }

    pub fn sample_count(mut self, count: u32) -> Self {
        self.desc.sample_count = count;
        self
    }

    pub fn usage(mut self, usage: wgpu::TextureUsages) -> Self {
        self.desc.usage = usage;
        self
    }

    pub fn format(mut self, format: wgpu::TextureFormat) -> Self {
        self.desc.format = format;
        self
    }

    pub fn build(&self, device: &wgpu::Device) -> wgpu::Texture {
        device.create_texture(&self.desc)
    }
}
