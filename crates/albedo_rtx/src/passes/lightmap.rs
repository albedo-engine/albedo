use albedo_backend::gpu;
use wgpu::{BindGroup, BindingType};

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

    pub fn new(device: &wgpu::Device) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
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
                    ty: BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
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
            "lightmap.vert.spv"
        )));
        let fg_module = device.create_shader_module(wgpu::include_spirv!(concat!(
            "..",
            path_separator!(),
            "shaders",
            path_separator!(),
            "spirv",
            path_separator!(),
            "lightmap.frag.spv"
        )));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Lightmap Pipeline"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let position = wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x4,
            shader_location: 0,
            offset: 0,
        };

        let normal = wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x4,
            shader_location: 1,
            offset: position.format.size(),
        };

        let layout = wgpu::VertexBufferLayout {
            array_stride: position.format.size() + normal.format.size(),
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[position, normal],
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Lightmap Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vx_module,
                entry_point: "main",
                buffers: &[layout],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fg_module,
                entry_point: "main",
                targets: &[Some(wgpu::TextureFormat::Rgba32Float.into())],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
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
        global_uniforms: &gpu::Buffer<uniforms::PerDrawUniforms>,
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
        vertex_count: u32,
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
        pass.set_bind_group(0, bind_group, &[]);
        /* Quickly hacked together, will only work with 1 instance */
        pass.set_vertex_buffer(0, vertices.slice(0..));
        pass.set_index_buffer(indices.inner().slice(0..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..indices.count() as u32, 0, 0..instances.count() as u32);
    }
}
