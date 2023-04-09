#[path = "../example/mod.rs"]
mod example;
use example::{meshes, Example};
use meshes::Geometry;

use std::borrow::Cow;

use albedo_backend::{
    BindGroupLayoutBuilder, Buffer, BufferInitDescriptor, IndexBuffer, Primitive,
    RenderPipelineBuilder, TypedBuffer,
};

struct VertexData {
    primitive: Primitive,
}
struct PickingExample {
    vertex_data: VertexData,
}

impl VertexData {
    fn new(app: &example::App) -> Self {
        let cube = meshes::CubeGeometry::new();
        let primitive = Primitive::new(
            Buffer::new_with_data(
                &app.device,
                BufferInitDescriptor {
                    label: Some("Cube Positions"),
                    contents: cube.vertices(),
                    usage: wgpu::BufferUsages::VERTEX,
                },
            ),
            IndexBuffer::U16(TypedBuffer::new_with_data(
                &app.device,
                BufferInitDescriptor {
                    label: Some("Cube Indices"),
                    contents: cube.indices(),
                    usage: wgpu::BufferUsages::VERTEX,
                },
            )),
        );

        let bgl = BindGroupLayoutBuilder::new_with_size(1)
            .uniform_buffer(wgpu::ShaderStages::VERTEX, None)
            .build(&app.device);

        let pipeline_layout = app
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bgl],
                push_constant_ranges: &[],
            });

        let shader = app
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
            });

        // let pipeline = RenderPipelineBuilder::new(wgpu::VertexState {
        //     module: &shader,
        //     entry_point: "vs_main",
        //     buffers: primitive.vertices().inner(),
        // })

        VertexData { primitive }
    }
}

impl Example for PickingExample {
    fn new(app: &example::App) -> Self {
        PickingExample {
            vertex_data: VertexData::new(app),
        }
    }

    fn resize(&mut self, app: &example::App) {}

    fn update(&mut self, event: winit::event::WindowEvent) {}

    fn render(&mut self, app: &example::App, view: &wgpu::TextureView) {}
}

fn main() {
    println!("Hello World!");
    example::start::<PickingExample>();
}
