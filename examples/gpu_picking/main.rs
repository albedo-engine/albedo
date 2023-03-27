#[path = "../example/mod.rs"]
mod example;
use example::{meshes, Example};
use meshes::Geometry;

use albedo_backend::{Buffer, BufferInitDescriptor, Primitive};

struct PickingExample {
    primitive: Primitive,
}

impl Example for PickingExample {
    fn new(app: &example::App) -> Self {
        let cube = meshes::CubeGeometry::new();
        let primitive = Primitive::new(
            Buffer::new_with_data(
                &app.device,
                BufferInitDescriptor {
                    label: None,
                    contents: cube.vertices(),
                    usage: wgpu::BufferUsages::VERTEX,
                },
            ),
            cube.indices(),
        );

        PickingExample {}
    }

    fn resize(&mut self, app: &example::App) {}

    fn update(&mut self, event: winit::event::WindowEvent) {}

    fn render(&mut self, app: &example::App, view: &wgpu::TextureView) {}
}

fn main() {
    println!("Hello World!");
    example::start::<PickingExample>();
}
