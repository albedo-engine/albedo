#[path = "../example.rs"]
mod example;
use example::Example;

struct PickingExample {}

impl Example for PickingExample {
    fn new() -> Self {
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
