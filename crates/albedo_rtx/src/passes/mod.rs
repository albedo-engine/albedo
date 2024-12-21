mod a_trous;
mod accumulation;
mod blit_pass;
mod blit_texture_pass;
mod denoise;
mod intersector;
mod lightmap;
mod shading;
mod temporal_accumulation;
mod ray;

pub use accumulation::AccumulationPass;
pub use a_trous::ATrousPass;
pub use blit_pass::BlitPass;
pub use blit_texture_pass::BlitTexturePass;
pub use intersector::IntersectorPass;
pub use lightmap::LightmapPass;
pub use shading::{PrimaryRayPass,ShadingPass};
pub use ray::RayPass;
pub use denoise::*;
pub use temporal_accumulation::TemporalAccumulationPass;

pub(crate) const GBUFFER_READ_TY: wgpu::BindingType = wgpu::BindingType::Texture {
    multisampled: false,
    sample_type: wgpu::TextureSampleType::Uint,
    view_dimension: wgpu::TextureViewDimension::D2,
};
pub(crate) const GBUFFER_WRITE_TY: wgpu::BindingType = wgpu::BindingType::StorageTexture {
    format: wgpu::TextureFormat::Rgba32Uint,
    access: wgpu::StorageTextureAccess::WriteOnly,
    view_dimension: wgpu::TextureViewDimension::D2,
};
