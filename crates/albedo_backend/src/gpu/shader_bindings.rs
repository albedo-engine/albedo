pub const fn uniform_entry(
    binding: u32,
    visibility: wgpu::ShaderStages,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: uniform(),
        count: None,
    }
}

pub const fn buffer_entry(
    binding: u32,
    visibility: wgpu::ShaderStages,
    readonly: bool,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: buffer(readonly),
        count: None,
    }
}

pub const fn storage_texture2d_entry(
    binding: u32,
    visibility: wgpu::ShaderStages,
    format: wgpu::TextureFormat,
    access: wgpu::StorageTextureAccess,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: storage_texture2d(format, access),
        count: None,
    }
}

pub const fn texture1d(binding: u32, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Texture {
            multisampled: false,
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2Array,
        },
        count: None,
    }
}

pub const fn texture2d_entry(
    binding: u32,
    visibility: wgpu::ShaderStages,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: texture2d(),
        count: None,
    }
}

pub const fn texture2darray_entry(
    binding: u32,
    visibility: wgpu::ShaderStages,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Texture {
            multisampled: false,
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2Array,
        },
        count: None,
    }
}

pub const fn sampler_entry(
    binding: u32,
    visibility: wgpu::ShaderStages,
    filtering: bool,
) -> wgpu::BindGroupLayoutEntry {
    let ty = if filtering {
        sampler(wgpu::SamplerBindingType::Filtering)
    } else {
        sampler(wgpu::SamplerBindingType::NonFiltering)
    };
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty,
        count: None,
    }
}

pub const fn buffer(readonly: bool) -> wgpu::BindingType {
    wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Storage {
            read_only: readonly,
        },
        has_dynamic_offset: false,
        min_binding_size: None,
    }
}

pub const fn uniform() -> wgpu::BindingType {
    wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Uniform,
        has_dynamic_offset: false,
        min_binding_size: None,
    }
}

pub const fn sampler(binding_type: wgpu::SamplerBindingType) -> wgpu::BindingType {
    wgpu::BindingType::Sampler(binding_type)
}

pub const fn storage_texture2d(
    format: wgpu::TextureFormat,
    access: wgpu::StorageTextureAccess,
) -> wgpu::BindingType {
    wgpu::BindingType::StorageTexture {
        format,
        access,
        view_dimension: wgpu::TextureViewDimension::D2,
    }
}

pub const fn texture2d() -> wgpu::BindingType {
    wgpu::BindingType::Texture {
        multisampled: false,
        sample_type: wgpu::TextureSampleType::Float { filterable: true },
        view_dimension: wgpu::TextureViewDimension::D2,
    }
}
