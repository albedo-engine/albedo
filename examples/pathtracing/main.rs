use wgpu::{Device, PowerPreference};
use winit::{
    event::{self, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};
use albedo_rtx::accel::{BVHBuilder, SAHBuilder, BVH};
use albedo_rtx::{Renderer};

mod gltf_loader;
use gltf_loader::load_gltf;

struct App {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    window: winit::window::Window,
    event_loop: EventLoop<()>,
    surface: wgpu::Surface,
    queue: wgpu::Queue,
}

async fn setup() -> App {
    let event_loop = EventLoop::new();
    let mut builder = winit::window::WindowBuilder::new();
    builder = builder.with_title("Albedo Pathtracer");

    let window = builder.build(&event_loop).unwrap();

    let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
    let (size, surface) = unsafe {
        let size = window.inner_size();
        let surface = instance.create_surface(&window);
        (size, surface)
    };
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
        })
        .await
        .expect("No suitable GPU adapters found on the system!");

    let optional_features: wgpu::Features = wgpu::Features::default();
    let required_features: wgpu::Features = wgpu::Features::empty();
    let adapter_features: wgpu::Features = wgpu::Features::default();
    let needed_limits = wgpu::Limits {
        max_storage_buffers_per_shader_stage: 8,
        ..wgpu::Limits::default()
    } ;
    let trace_dir = std::env::var("WGPU_TRACE");

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: (optional_features & adapter_features) | required_features,
                limits: needed_limits,
            },
            trace_dir.ok().as_ref().map(std::path::Path::new),
        )
        .await
        .expect("Unable to find a suitable GPU adapter!");

    App {
        instance,
        adapter,
        device,
        window,
        event_loop,
        surface,
        queue,
    }
}

fn main() {

    let width = 640;
    let height = 480;

    let App {
        instance,
        adapter,
        device,
        window,
        event_loop,
        surface,
        queue,
    } = pollster::block_on(setup());
    let mut renderer= Renderer::new(&device);

    let mut sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
        format: adapter.get_swap_chain_preferred_format(&surface),
        width: width,
        height: height,
        present_mode: wgpu::PresentMode::Mailbox,
    };
    let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);

    let scene = load_gltf(&"./examples/pathtracing/assets/box.glb");

    let mut bvhs: Vec<BVH> = scene
        .meshes
        .iter()
        .map(|mesh| {
            let mut builder = SAHBuilder::new();
            builder.build(mesh).unwrap()
        })
        .collect();

    for n in &bvhs[0].nodes {
        println!("{}", n.aabb());
    }

    event_loop.run(move |event, _, control_flow| {
        // let _ = (&renderer, &app);
        match event {
            event::Event::RedrawRequested(_) => {
                let frame = match swap_chain.get_current_frame() {
                    Ok(frame) => frame,
                    Err(_) => {
                        swap_chain = device.create_swap_chain(&surface, &sc_desc);
                        swap_chain
                            .get_current_frame()
                            .expect("Failed to acquire next swap chain texture!")
                    }
                };
                renderer.render(&device, &frame.output, &queue);
            }
            _ => {}
        }
    });

}
