use std::borrow::Cow;

use albedo_backend::data::ShaderCache;
use albedo_backend::gpu;
use wgpu::BindGroup;

use crate::macros::path_separator;
use crate::uniforms;

pub struct LightmapPass {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
}

impl LightmapPass {
    const INSTANCE_BINDING: u32 = 0;
    const NODE_BINDING: u32 = 1;
    const INDEX_BINDING: u32 = 2;
    const VERTEX_BINDING: u32 = 3;
    const PER_DRAW_STRUCT_BINDING: u32 = 4;

    pub fn new(
        device: &wgpu::Device,
        processor: &ShaderCache,
        target_format: wgpu::TextureFormat,
    ) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Lightmap Pass Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: Self::INSTANCE_BINDING,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: Self::NODE_BINDING,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: Self::INDEX_BINDING,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: Self::VERTEX_BINDING,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: Self::PER_DRAW_STRUCT_BINDING,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let vx_module = processor
            .compile_vertex(include_str!(concat!(
                "..",
                path_separator!(),
                "..",
                path_separator!(),
                "shaders",
                path_separator!(),
                "lightmap.vert"
            )))
            .unwrap();
        let vx_module: wgpu::ShaderModule =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Lightmap Vertex Shaderr"),
                source: wgpu::ShaderSource::Naga(Cow::Owned(vx_module)),
            });
        let fg_module = processor
            .compile_fragment(include_str!(concat!(
                "..",
                path_separator!(),
                "..",
                path_separator!(),
                "shaders",
                path_separator!(),
                "lightmap.frag"
            )))
            .unwrap();
        let fg_module: wgpu::ShaderModule =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Lightmap Fragment Shader"),
                source: wgpu::ShaderSource::Naga(Cow::Owned(fg_module)),
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Lightmap Pipeline"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let layout_builder = gpu::VertexBufferLayoutBuilder::new(2)
            .auto_attribute(wgpu::VertexFormat::Float32x4)
            .auto_attribute(wgpu::VertexFormat::Float32x4);
        let layout = layout_builder.build(None);

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Lightmap Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vx_module,
                entry_point: Some("main"),
                buffers: &[layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &fg_module,
                entry_point: Some("main"),
                targets: &[Some(target_format.into())],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        LightmapPass {
            bind_group_layout,
            pipeline,
        }
    }

    pub fn create_frame_bind_groups(
        &self,
        device: &wgpu::Device,
        instances: &gpu::Buffer<uniforms::Instance>,
        nodes: &wgpu::Buffer,
        indices: &gpu::Buffer<u32>,
        vertices: &wgpu::Buffer,
        global_uniforms: gpu::UniformBufferSlice<uniforms::PerDrawUniforms>,
    ) -> BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Lightmap Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: Self::INSTANCE_BINDING,
                    resource: instances.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: Self::NODE_BINDING,
                    resource: nodes.as_entire_binding(),
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
                    binding: Self::PER_DRAW_STRUCT_BINDING,
                    resource: global_uniforms.as_entire_binding(),
                },
            ],
        })
    }

    pub fn draw(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        bind_group: &BindGroup,
        instances: &gpu::Buffer<uniforms::Instance>,
        indices: &gpu::Buffer<u32>,
        vertices: &wgpu::Buffer,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, bind_group, &[]);
        /* Quickly hacked together, will only work with 1 instance */
        pass.set_vertex_buffer(0, vertices.slice(0..));
        pass.set_index_buffer(indices.inner().slice(0..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..indices.count() as u32, 0, 0..instances.count() as u32);
    }
}
