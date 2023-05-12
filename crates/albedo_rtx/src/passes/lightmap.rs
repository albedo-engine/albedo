use albedo_backend::gpu;

use crate::macros::path_separator;
use crate::uniforms::{Instance, Light, Material, PerDrawUniforms};
use crate::{SceneLayout};

pub struct LightmapPass {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
}

impl LightmapPass {
    pub fn new(device: &wgpu::Device, scene_layout: &SceneLayout, target_format: wgpu::TextureFormat) -> Self {
        let scene_layout = &scene_layout.layout;
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[]
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
            bind_group_layouts: &[&scene_layout, &bind_group_layout],
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
                entry_point: "main",
                buffers: &[layout],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fg_module,
                entry_point: "main",
                targets: &[Some(target_format.into())],
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

    pub fn create_bind_groups(
        &self,
        device: &wgpu::Device,
        scene_layout: &SceneLayout,
        instances: &gpu::Buffer<Instance>,
        nodes: &wgpu::Buffer,
        indices: &gpu::Buffer<u32>,
        vertices: &wgpu::Buffer,
        lights: &gpu::Buffer<Light>,
        materials: &gpu::Buffer<Material>,
        global_uniforms: &gpu::Buffer<PerDrawUniforms>,
    ) -> [wgpu::BindGroup; 2] {
        let scene_bind_group = scene_layout.bind_group(1)
            .uniforms(global_uniforms)
            .instances(instances).nodes(nodes).indices(indices).vertices(vertices)
            .lights(lights).materials(materials)
            .create(device);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Lightmap Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[],
        });
        [scene_bind_group, bind_group]
    }

    pub fn draw(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        bind_groups: &[wgpu::BindGroup; 2],
        instances: &gpu::Buffer<Instance>,
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
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });
        pass.set_pipeline(&self.pipeline);
        for i in 0..bind_groups.len() {
            pass.set_bind_group(i as u32, &bind_groups[i], &[]);
        }
        /* Quickly hacked together, will only work with 1 instance */
        pass.set_vertex_buffer(0, vertices.slice(0..));
        pass.set_index_buffer(indices.inner().slice(0..), wgpu::IndexFormat::Uint32);
        pass.draw_indexed(0..indices.count() as u32, 0, 0..instances.count() as u32);
    }
}
