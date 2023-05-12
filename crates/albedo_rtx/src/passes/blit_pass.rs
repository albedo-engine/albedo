use albedo_backend::gpu;
use wgpu;

use crate::macros::path_separator;
use crate::{uniforms, SceneLayout};

pub struct BlitPass {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
}

impl BlitPass {
    const TEXTURE_SAMPLER_BINDING: u32 = 0;
    const TEXTURE_BINDING: u32 = 1;

    pub fn new(device: &wgpu::Device, scene_layout: &SceneLayout, swap_chain_format: wgpu::TextureFormat) -> Self {
        let scene_layout = &scene_layout.layout;
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: Self::TEXTURE_SAMPLER_BINDING,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    // ty: BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: Self::TEXTURE_BINDING,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        // @todo: Should be filterable.
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });

        let vx_module = device.create_shader_module(wgpu::include_spirv!(concat!(
            "..",
            path_separator!(),
            "shaders",
            path_separator!(),
            "spirv",
            path_separator!(),
            "blitting.vert.spv"
        )));
        let fg_module = device.create_shader_module(wgpu::include_spirv!(concat!(
            "..",
            path_separator!(),
            "shaders",
            path_separator!(),
            "spirv",
            path_separator!(),
            "blitting.frag.spv"
        )));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Blit Pipeline"),
            bind_group_layouts: &[&scene_layout, &bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Blit Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vx_module,
                entry_point: "main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fg_module,
                entry_point: "main",
                targets: &[Some(swap_chain_format.into())],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        BlitPass {
            bind_group_layout,
            pipeline,
        }
    }

    pub fn create_bind_groups(
        &self,
        device: &wgpu::Device,
        scene_layout: &SceneLayout,
        view: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
        global_uniforms: &gpu::Buffer<uniforms::PerDrawUniforms>,
    ) -> [wgpu::BindGroup; 2] {
        let scene_bind_group = scene_layout.bind_group(1).uniforms(global_uniforms).create(device);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Blit Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: Self::TEXTURE_SAMPLER_BINDING,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
                wgpu::BindGroupEntry {
                    binding: Self::TEXTURE_BINDING,
                    resource: wgpu::BindingResource::TextureView(view),
                },
            ],
        });
        [scene_bind_group, bind_group]
    }

    pub fn draw(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        bind_groups: &[wgpu::BindGroup; 2],
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.9,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
        pass.set_pipeline(&self.pipeline);
        for i in 0..bind_groups.len() {
            pass.set_bind_group(i as u32, &bind_groups[i], &[]);
        }
        pass.draw(0..3, 0..1);
    }
}
