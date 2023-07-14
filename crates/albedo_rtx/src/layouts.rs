use std::ops::Deref;

use crate::uniforms;
use albedo_backend::gpu;

pub struct RTSceneBindGroupLayout(wgpu::BindGroupLayout);

impl RTSceneBindGroupLayout {
    const INSTANCE_BINDING: u32 = 0;
    const NODE_BINDING: u32 = 1;
    const INDEX_BINDING: u32 = 2;
    const VERTEX_BINDING: u32 = 3;
    const LIGHT_BINDING: u32 = 4;
    const MATERIAL_BINDING: u32 = 5;
    const TEXTURE_PROBE_BINDING: u32 = 6;
    const TEXTURE_INFO_BINDING: u32 = 7;
    const TEXTURE_ATLAS_BINDING: u32 = 8;
    const SAMPLER_BINDING: u32 = 9;
    const SAMPLER_LINEAR_BINDING: u32 = 10;

    pub fn new(device: &wgpu::Device) -> Self {
        let inner = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Radiance Estimator Base Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: Self::NODE_BINDING,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: Self::INSTANCE_BINDING,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: Self::INDEX_BINDING,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: Self::VERTEX_BINDING,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: Self::LIGHT_BINDING,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: Self::MATERIAL_BINDING,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: Self::TEXTURE_PROBE_BINDING,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: Self::TEXTURE_INFO_BINDING,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Uint,
                        view_dimension: wgpu::TextureViewDimension::D1,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: Self::TEXTURE_ATLAS_BINDING,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: Self::SAMPLER_BINDING,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: Self::SAMPLER_LINEAR_BINDING,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        Self { 0: inner }
    }

    pub fn create_geometry_bindgroup(
        &self,
        device: &wgpu::Device,
        nodes: gpu::StorageBufferSlice<albedo_bvh::BVHNode>,
        instances: gpu::StorageBufferSlice<uniforms::Instance>,
        indices: gpu::StorageBufferSlice<u32>,
        vertices: gpu::StorageBufferSlice<uniforms::Vertex>,
        lights: gpu::StorageBufferSlice<uniforms::Light>,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Radiance Estimator Base Bind Group"),
            layout: &self.0,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: Self::NODE_BINDING,
                    resource: nodes.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: Self::INSTANCE_BINDING,
                    resource: instances.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: Self::INDEX_BINDING,
                    resource: indices.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: Self::VERTEX_BINDING,
                    resource: vertices.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: Self::LIGHT_BINDING,
                    resource: lights.as_entire_binding(),
                },
            ],
        })
    }

    pub fn create_bindgroup(
        &self,
        device: &wgpu::Device,
        nodes: gpu::StorageBufferSlice<albedo_bvh::BVHNode>,
        instances: gpu::StorageBufferSlice<uniforms::Instance>,
        indices: gpu::StorageBufferSlice<u32>,
        vertices: gpu::StorageBufferSlice<uniforms::Vertex>,
        lights: gpu::StorageBufferSlice<uniforms::Light>,
        materials: gpu::StorageBufferSlice<uniforms::Material>,
        probe: &wgpu::TextureView,
        textures_info: &wgpu::TextureView,
        texture_atlas: &wgpu::TextureView,
        sampler_nearest: &wgpu::Sampler,
        sampler_linear: &wgpu::Sampler,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Radiance Estimator Base Bind Group"),
            layout: &self.0,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: Self::NODE_BINDING,
                    resource: nodes.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: Self::INSTANCE_BINDING,
                    resource: instances.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: Self::INDEX_BINDING,
                    resource: indices.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: Self::VERTEX_BINDING,
                    resource: vertices.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: Self::LIGHT_BINDING,
                    resource: lights.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: Self::MATERIAL_BINDING,
                    resource: materials.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: Self::TEXTURE_PROBE_BINDING,
                    resource: wgpu::BindingResource::TextureView(probe),
                },
                wgpu::BindGroupEntry {
                    binding: Self::TEXTURE_INFO_BINDING,
                    resource: wgpu::BindingResource::TextureView(textures_info),
                },
                wgpu::BindGroupEntry {
                    binding: Self::TEXTURE_ATLAS_BINDING,
                    resource: wgpu::BindingResource::TextureView(texture_atlas),
                },
                wgpu::BindGroupEntry {
                    binding: Self::SAMPLER_BINDING,
                    resource: wgpu::BindingResource::Sampler(sampler_nearest),
                },
                wgpu::BindGroupEntry {
                    binding: Self::SAMPLER_LINEAR_BINDING,
                    resource: wgpu::BindingResource::Sampler(sampler_linear),
                },
            ],
        })
    }

    pub fn inner(&self) -> &wgpu::BindGroupLayout {
        &self.0
    }
}

impl Deref for RTSceneBindGroupLayout {
    type Target = wgpu::BindGroupLayout;

    fn deref(&self) -> &Self::Target {
        self.inner()
    }
}
