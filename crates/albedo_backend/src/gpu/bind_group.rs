pub struct BindGroupLayoutBuilder<'a> {
    entries: Vec<wgpu::BindGroupLayoutEntry>,
    label: wgpu::Label<'a>,
}

impl<'a> BindGroupLayoutBuilder<'a> {
    pub fn new_with_size(size: usize) -> BindGroupLayoutBuilder<'a> {
        BindGroupLayoutBuilder {
            entries: Vec::with_capacity(size),
            label: None,
        }
    }

    pub fn build(self, device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: self.label,
            entries: self.entries.as_ref(),
        })
    }

    pub fn label(mut self, label: wgpu::Label<'a>) -> Self {
        self.label = label;
        self
    }

    pub fn storage_buffer(self, visibility: wgpu::ShaderStages, read_only: bool) -> Self {
        self.insert(
            visibility,
            wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
        )
    }

    pub fn uniform_buffer(
        self,
        visibility: wgpu::ShaderStages,
        min_binding_size: Option<wgpu::BufferSize>,
    ) -> Self {
        self.insert(
            visibility,
            wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size,
            },
        )
    }

    pub fn texture(
        self,
        visibility: wgpu::ShaderStages,
        sample_type: wgpu::TextureSampleType,
        view_dimension: wgpu::TextureViewDimension,
    ) -> Self {
        self.insert(
            visibility,
            wgpu::BindingType::Texture {
                multisampled: false,
                sample_type,
                view_dimension,
            },
        )
    }

    pub fn sampler(
        self,
        visibility: wgpu::ShaderStages,
        binding_type: wgpu::SamplerBindingType,
    ) -> Self {
        self.insert(visibility, wgpu::BindingType::Sampler(binding_type))
    }

    fn insert(mut self, visibility: wgpu::ShaderStages, ty: wgpu::BindingType) -> Self {
        let binding = self.entries.len() as u32;
        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty,
            count: None,
        });
        self
    }
}
