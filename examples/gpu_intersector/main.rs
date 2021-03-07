use albedo_backend::{shader_bindings, GPUBuffer, UniformBuffer};

use albedo_rtx::passes::{BlitPass, GPUIntersector, GPURadianceEstimator, GPURayGenerator};
use albedo_rtx::renderer::resources;

use winit::{
    event::{self},
    event_loop::{EventLoop},
};

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
    size: winit::dpi::PhysicalSize<u32>
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
    };
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
        size
    }
}

fn main() {
    let App {
        instance,
        adapter,
        device,
        window,
        event_loop,
        surface,
        queue,
        size
    } = pollster::block_on(setup());

    println!("Window Size = {}x{}", size.width, size.height);

    let mut sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
        format: adapter.get_swap_chain_preferred_format(&surface),
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Mailbox,
    };
    let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);

    let scene = load_gltf(&"./examples/pathtracing/assets/box.glb");

    //// Scene Info

    println!("Materials = [");
    for mat in &scene.materials {
        println!("\t( color: {} ),", mat.color);
    }
    println!("]");

    let lights = vec![resources::LightGPU::from_matrix(
        glam::Mat4::from_scale_rotation_translation(
            glam::Vec3::new(1.0, 1.0, 1.0),
            glam::Quat::from_rotation_x(1.5),
            glam::Vec3::new(0.0, 1.5, 0.75),
        )
    )];

    let mut camera = resources::CameraGPU::new();
    camera.origin = glam::Vec3::new(0.0, 0.0, 5.0);

    let pixel_count = size.width * size.height;

    let render_target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Render Target"),
        size: wgpu::Extent3d { width: size.width, height: size.height, depth: 1, },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba32Float,
        usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::STORAGE,
    });
    let render_target_view = render_target.create_view(&wgpu::TextureViewDescriptor::default());
    let render_target_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    let filtered_sampler_2d = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    let mut global_uniforms = resources::GlobalUniformsGPU::new();
    let mut global_uniforms_buffer = UniformBuffer::new(&device);

    let mut camera_buffer = UniformBuffer::new(&device);
    camera_buffer.update(&queue, &camera);

    let mut instance_buffer = GPUBuffer::from_data(&device, &scene.instances);
    instance_buffer.update(&queue, &scene.instances);

    let mut materials_buffer = GPUBuffer::from_data(&device, &scene.materials);
    materials_buffer.update(&queue, &scene.materials);

    let mut bvh_buffer = GPUBuffer::from_data(&device, &scene.node_buffer);
    bvh_buffer.update(&queue, &scene.node_buffer);

    let mut index_buffer = GPUBuffer::from_data(&device, &scene.index_buffer);
    index_buffer.update(&queue, &scene.index_buffer);

    let mut vertex_buffer = GPUBuffer::from_data(&device, &scene.vertex_buffer);
    vertex_buffer.update(&queue, &scene.vertex_buffer);

    let mut light_buffer = GPUBuffer::from_data(&device, &lights);
    light_buffer.update(&queue, &lights);

    let mut scene_buffer = UniformBuffer::new(&device);
    scene_buffer.update(
        &queue,
        &resources::SceneSettingsGPU {
            light_count: lights.len() as u32,
            instance_count: scene.instances.len() as u32,
        },
    );

    //// Load HDRi enviromment.

    let file_reader = std::io::BufReader::new(
        std::fs::File::open("./examples/gpu_intersector/assets/uffizi-large.hdr").unwrap()
    );
    let decoder = image::hdr::HdrDecoder::new(file_reader).unwrap();
    let metadata = decoder.metadata();
    let image_data = decoder.read_image_native().unwrap();
    let image_data_raw = unsafe {
        std::slice::from_raw_parts(image_data.as_ptr() as *const u8, image_data.len() * std::mem::size_of::<image::hdr::Rgbe8Pixel>())
    };

    let probe_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Cubemap"),
        size: wgpu::Extent3d {
            width: metadata.width,
            height: metadata.height,
            depth: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
    });
    let probe_view = probe_texture.create_view(&wgpu::TextureViewDescriptor::default());
    queue.write_texture(
        wgpu::TextureCopyView {
            texture: &probe_texture,
            mip_level: 0,
            origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
        },
        image_data_raw,
        wgpu::TextureDataLayout {
            offset: 0,
            bytes_per_row: std::mem::size_of::<image::hdr::Rgbe8Pixel>() as u32 * metadata.width,
            rows_per_image: 0,
        },
        wgpu::Extent3d {
            width: metadata.width,
            height: metadata.height,
            depth: 1,
        },
    );

    let ray_buffer = GPUBuffer::new_with_count(&device, pixel_count as usize);
    let intersection_buffer = GPUBuffer::new_with_count(&device, pixel_count as usize);

    let mut generate_ray_pass = GPURayGenerator::new(&device);
    let mut intersector_pass = GPUIntersector::new(&device);
    let mut shade_pass = GPURadianceEstimator::new(&device);
    let mut blit_pass = BlitPass::new(&device, sc_desc.format);

    generate_ray_pass.bind_buffers(&device, &ray_buffer, &camera_buffer);
    intersector_pass.bind_buffers(
        &device,
        &intersection_buffer,
        &instance_buffer,
        &bvh_buffer,
        &index_buffer,
        &vertex_buffer,
        &light_buffer,
        &ray_buffer,
        &scene_buffer,
    );
    shade_pass.bind_buffers(
        &device,
        &ray_buffer,
        &intersection_buffer,
        &instance_buffer,
        &index_buffer,
        &vertex_buffer,
        &light_buffer,
        &materials_buffer,
        &scene_buffer,
        &probe_view,
        &filtered_sampler_2d
    );
    shade_pass.bind_target(&device, &render_target_view, &global_uniforms_buffer);
    blit_pass.bind(&device, &render_target_view, &render_target_sampler);

    let nb_bounces = 3 as usize;

    event_loop.run(move |event, _, control_flow| {
        // let _ = (&renderer, &app);
        match event {
            event::Event::MainEventsCleared => {
                window.request_redraw();
            },
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

                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                generate_ray_pass.run(&mut encoder, size.width, size.height);

                for _ in 0..nb_bounces {
                    global_uniforms_buffer.update(&queue, &global_uniforms);

                    intersector_pass.run(&device, &mut encoder, size.width, size.height);
                    shade_pass.run(&mut encoder, size.width, size.height);

                    global_uniforms.frame_count = global_uniforms.frame_count + 1;
                }
                blit_pass.run(&frame.output, &queue, &mut encoder);
                queue.submit(Some(encoder.finish()));

                global_uniforms.frame_count = global_uniforms.frame_count + 1;
            }
            _ => {}
        }
    });
}
