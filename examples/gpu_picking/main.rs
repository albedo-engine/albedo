#[path = "../example/mod.rs"]
mod example;
use example::{
    meshes::{self, Vertex},
    Example,
};
use meshes::Geometry;

use std::borrow::Cow;

use albedo_backend::{
    BindGroupLayoutBuilder, BufferInitDescriptor, IndexBuffer, RenderPipelineBuilder,
    StorageBuffer, UniformBuffer, VertexBufferLayoutBuilder,
};

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Vertex3D {
    position: [f32; 4],
    normal: [f32; 4],
}
unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

impl Vertex for Vertex3D {
    fn as_vertex_formats() -> &'static [wgpu::VertexFormat] {
        &[wgpu::VertexFormat::Float32x4, wgpu::VertexFormat::Float32x4]
    }
    fn set_position(&mut self, pos: &[f32; 3]) {
        self.position.copy_from_slice(&pos[0..3])
    }
    fn set_normal(&mut self, normal: &[f32; 3]) {
        self.normal.copy_from_slice(&normal[0..3])
    }
}

struct PickingExample {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    vertex_buffer: StorageBuffer<Vertex>,
    index_buffer: IndexBuffer,
}

// @todo: Create a UniformBlock derive
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Uniforms {
    transform: glam::Mat4,
}
unsafe impl bytemuck::Pod for Uniforms {}
unsafe impl bytemuck::Zeroable for Uniforms {}

impl Example for PickingExample {
    fn new(app: &example::App) -> Self {
        let bgl = BindGroupLayoutBuilder::new_with_size(1)
            .uniform_buffer(wgpu::ShaderStages::VERTEX, None)
            .build(&app.device);

        let shader = app
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
            });

        let vertex_buffer_layout = VertexBufferLayoutBuilder::new(2)
            .auto_attribute(wgpu::VertexFormat::Float32x4)
            .auto_attribute(wgpu::VertexFormat::Float32x2);

        let pipeline_layout = app
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bgl],
                push_constant_ranges: &[],
            });

        let pipeline = RenderPipelineBuilder::new(wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[vertex_buffer_layout.build(None)],
        })
        .layout(&pipeline_layout)
        .fragment(Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(app.surface_config.format.into())],
        }))
        .build(&app.device);

        let cube = meshes::CubeGeometry::new();
        let vertex_buffer = StorageBuffer::sized_with_data(
            &app.device,
            cube.vertices(),
            Some(BufferInitDescriptor::new(
                Some("Cube Positions"),
                wgpu::BufferUsages::VERTEX,
            )),
        );
        let index_buffer = IndexBuffer::with_data_16(
            &app.device,
            cube.indices(),
            Some(BufferInitDescriptor::new(
                Some("Cube Indices"),
                wgpu::BufferUsages::VERTEX,
            )),
        );

        let aspect_ratio = app.surface_config.width as f32 / app.surface_config.height as f32;

        let mut uniforms = Uniforms {
            transform: glam::Mat4::IDENTITY,
        };
        uniforms.transform = glam::Mat4::perspective_rh_gl(0.38, aspect_ratio, 0.01, 100.0)
            * glam::Mat4::from_translation(glam::Vec3::new(0.0, 0.0, -10.0));
        let uniform_buffer = UniformBuffer::sized_with_data(&app.device, &uniforms, None);

        let bind_group = app.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("Bind Group"),
        });
        // let vertex_data = VertexData::new(app);

        PickingExample {
            pipeline,
            bind_group,
            vertex_buffer,
            index_buffer,
        }
    }

    fn resize(&mut self, app: &example::App) {}

    fn update(&mut self, event: winit::event::WindowEvent) {}

    fn render(&mut self, app: &example::App, view: &wgpu::TextureView) {
        let mut encoder = app
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            rpass.push_debug_group("Prepare data for draw.");
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_index_buffer(
                self.index_buffer.inner().slice(..),
                wgpu::IndexFormat::Uint16,
            );
            rpass.set_vertex_buffer(0, self.vertex_buffer.inner().slice(..));
            rpass.pop_debug_group();
            rpass.insert_debug_marker("Draw!");
            rpass.draw_indexed(0..self.index_buffer.count() as u32, 0, 0..1);
        }

        app.queue.submit(Some(encoder.finish()));
    }
}

fn main() {
    println!("Hello World!");
    example::start::<PickingExample>();
}
