// common binding types mapped to glsl type names
// pub fn buffer(readonly: bool) -> wgpu::BindingType {
//     wgpu::BindingType::Buffer {
//         ty: wgpu::BufferBindingType::Storage {
//             read_only: readonly
//         },
//         has_dynamic_offset: false,
//         min_binding_size: None
//     }
// }

pub const fn buffer(readonly: bool) -> wgpu::BindingType {
    wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Storage {
            read_only: readonly
        },
        has_dynamic_offset: false,
        min_binding_size: None
    }
}

pub const fn uniform() -> wgpu::BindingType {
    wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Uniform,
        has_dynamic_offset: false,
        min_binding_size: None
    }
}

pub const fn sampler(filtering: bool) -> wgpu::BindingType {
    wgpu::BindingType::Sampler {
        comparison: false,
        filtering: filtering
    }
}

pub const fn storage_texture2d(format: wgpu::TextureFormat, access: wgpu::StorageTextureAccess) -> wgpu::BindingType {
    wgpu::BindingType::StorageTexture {
        format,
        access,
        view_dimension: wgpu::TextureViewDimension::D2
    }
}

pub const fn texture2d() -> wgpu::BindingType {
    wgpu::BindingType::Texture {
        multisampled: false,
        sample_type: wgpu::TextureSampleType::Float {
            filterable: true
        },
        view_dimension: wgpu::TextureViewDimension::D2
    }
}

// pub fn texture2DArray() -> wgpu::BindingType {
//     wgpu::BindingType::SampledTexture {
//         multisampled: false,
//         component_type: wgpu::TextureComponentType::Float,
//         dimension: wgpu::TextureViewDimension::D2Array,
//     }
// }

// pub fn itexture2D() -> wgpu::BindingType {
//     wgpu::BindingType::SampledTexture {
//         multisampled: false,
//         component_type: wgpu::TextureComponentType::Sint,
//         dimension: wgpu::TextureViewDimension::D2,
//     }
// }

// pub fn utexture2D() -> wgpu::BindingType {
//     wgpu::BindingType::SampledTexture {
//         multisampled: false,
//         component_type: wgpu::TextureComponentType::Uint,
//         dimension: wgpu::TextureViewDimension::D2,
//     }
// }

// pub fn texture3D() -> wgpu::BindingType {
//     wgpu::BindingType::SampledTexture {
//         multisampled: false,
//         component_type: wgpu::TextureComponentType::Float,
//         dimension: wgpu::TextureViewDimension::D3,
//     }
// }

// pub fn itexture3D() -> wgpu::BindingType {
//     wgpu::BindingType::SampledTexture {
//         multisampled: false,
//         component_type: wgpu::TextureComponentType::Sint,
//         dimension: wgpu::TextureViewDimension::D3,
//     }
// }

// pub fn utexture3D() -> wgpu::BindingType {
//     wgpu::BindingType::SampledTexture {
//         multisampled: false,
//         component_type: wgpu::TextureComponentType::Uint,
//         dimension: wgpu::TextureViewDimension::D3,
//     }
// }

// pub fn textureCube() -> wgpu::BindingType {
//     wgpu::BindingType::SampledTexture {
//         multisampled: false,
//         component_type: wgpu::TextureComponentType::Float,
//         dimension: wgpu::TextureViewDimension::Cube,
//     }
// }

// pub fn image2D(format: wgpu::TextureFormat, readonly: bool) -> wgpu::BindingType {
//     wgpu::BindingType::StorageTexture {
//         dimension: wgpu::TextureViewDimension::D2,
//         format: format,
//         readonly,
//     }
// }

// pub fn image2DArray(format: wgpu::TextureFormat, readonly: bool) -> wgpu::BindingType {
//     wgpu::BindingType::StorageTexture {
//         dimension: wgpu::TextureViewDimension::D2Array,
//         format: format,
//         readonly,
//     }
// }

// pub fn iimage2D(format: wgpu::TextureFormat, readonly: bool) -> wgpu::BindingType {
//     wgpu::BindingType::StorageTexture {
//         dimension: wgpu::TextureViewDimension::D2,
//         format: format,
//         readonly,
//     }
// }

// pub fn uimage2D(format: wgpu::TextureFormat, readonly: bool) -> wgpu::BindingType {
//     wgpu::BindingType::StorageTexture {
//         dimension: wgpu::TextureViewDimension::D2,
//         format: format,
//         readonly,
//     }
// }

// pub fn image3D(format: wgpu::TextureFormat, readonly: bool) -> wgpu::BindingType {
//     wgpu::BindingType::StorageTexture {
//         dimension: wgpu::TextureViewDimension::D3,
//         format: format,
//         readonly,
//     }
// }

// pub fn iimage3D(format: wgpu::TextureFormat, readonly: bool) -> wgpu::BindingType {
//     wgpu::BindingType::StorageTexture {
//         dimension: wgpu::TextureViewDimension::D3,
//         format: format,
//         readonly,
//     }
// }

// pub fn uimage3D(format: wgpu::TextureFormat, readonly: bool) -> wgpu::BindingType {
//     wgpu::BindingType::StorageTexture {
//         dimension: wgpu::TextureViewDimension::D3,
//         format: format,
//         readonly,
//     }
// }
