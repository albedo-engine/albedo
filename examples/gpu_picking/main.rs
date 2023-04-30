#[path = "../example/mod.rs"]
mod example;
use example::{
    meshes::{self},
    Example,
};
use meshes::Geometry;

use std::borrow::Cow;

use albedo_backend::gpu;
use albedo_backend::gpu::{AsBuffer, AsVertexBufferLayout};

use albedo_backend::mesh::shapes::Shape;
use albedo_backend::mesh::*;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Vertex3D {
    position: [f32; 4],
    normal: [f32; 4],
}
unsafe impl bytemuck::Pod for Vertex3D {}
unsafe impl bytemuck::Zeroable for Vertex3D {}

impl AsVertexFormat for Vertex3D {
    fn as_vertex_formats() -> &'static [AttributeDescriptor] {
        static AttributeDescriptor: [AttributeDescriptor; 2] = [
            AttributeDescriptor {
                id: AttributeId::POSITION,
                format: wgpu::VertexFormat::Float32x4,
            },
            AttributeDescriptor {
                id: AttributeId::NORMAL,
                format: wgpu::VertexFormat::Float32x4,
            },
        ];
        &AttributeDescriptor
    }
}

impl Vertex for Vertex3D {
    fn set_position(&mut self, pos: &[f32; 3]) {
        self.position[..3].copy_from_slice(&pos[..3])
    }
    fn set_normal(&mut self, normal: &[f32; 3]) {
        self.normal[..3].copy_from_slice(&normal[..3])
    }
}

struct PickingExample {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    vertex_buffers: Vec<gpu::BufferHandle>,
    index_buffer: gpu::IndexBuffer,
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
        let bgl = gpu::BindGroupLayoutBuilder::new_with_size(1)
            .uniform_buffer(wgpu::ShaderStages::VERTEX, None)
            .build(&app.device);

        let shader = app
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
            });

        let primitive = shapes::Cube::new(1.0).to_interleaved_primitive::<Vertex3D>();
        let vertex_buffer_layout = primitive.as_vertex_buffer_layout();

        let pipeline_layout = app
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bgl],
                push_constant_ranges: &[],
            });

        let pipeline = gpu::RenderPipelineBuilder::new(wgpu::VertexState {
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
        let vertex_buffers = primitive.as_gpu_buffer(
            &app.device,
            gpu::BufferInitDescriptor::new(
                Some("Cube Positions"),
                wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
            ),
        );
        // let vertex_buffer = gpu::StorageBuffer::sized_with_data(
        //     &app.device,
        //     cube.vertices(),
        //     Some(gpu::BufferInitDescriptor::new(
        //         Some("Cube Positions"),
        //         wgpu::BufferUsages::VERTEX,
        //     )),
        // );
        let index_buffer = gpu::IndexBuffer::with_data_16(
            &app.device,
            cube.indices(),
            Some(gpu::BufferInitDescriptor::new(
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
        let uniform_buffer = gpu::UniformBuffer::sized_with_data(&app.device, &uniforms, None);

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
            vertex_buffers,
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
            for i in 0..self.vertex_buffers.len() {
                rpass.set_vertex_buffer(i as u32, self.vertex_buffers[i].inner().slice(..));
            }
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
