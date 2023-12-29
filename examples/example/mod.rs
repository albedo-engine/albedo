use winit::{
    event::{self, WindowEvent},
    event_loop::ControlFlow,
};

mod async_exec;
use async_exec::Spawner;

type EventLoop = winit::event_loop::EventLoop<()>;

pub struct App {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub window: winit::window::Window,
    pub surface: wgpu::Surface,
    pub queue: wgpu::Queue,
    pub surface_config: wgpu::SurfaceConfiguration,
    #[cfg(not(target_arch = "wasm32"))]
    pub executor: Spawner<'static>,
    #[cfg(target_arch = "wasm32")]
    pub executor: Spawner,
    pub mouse: glam::Vec2,
}

pub async fn initialize<E>(title: &str) -> (EventLoop, App) {
    let event_loop = winit::event_loop::EventLoopBuilder::with_user_event().build();
    let mut builder = winit::window::WindowBuilder::new();
    builder = builder.with_title(title);

    let window = builder.build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::{prelude::*, JsCast};
        use winit::platform::web::WindowExtWebSys;

        let canvas = window.canvas();
        // On wasm, append the canvas to the document body
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| body.append_child(&web_sys::Element::from(canvas)).ok())
            .expect("couldn't append canvas to document body");
    }

    let dx12_shader_compiler = wgpu::util::dx12_shader_compiler_from_env().unwrap_or_default();
    let gles_minor_version = wgpu::util::gles_minor_version_from_env().unwrap_or_default();

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        dx12_shader_compiler,
        gles_minor_version,
        flags: wgpu::InstanceFlags::from_build_config().with_env(),
    });
    let surface = unsafe { instance.create_surface(&window) }.unwrap();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .expect("No suitable GPU adapters found on the system!");

    let optional_features: wgpu::Features = wgpu::Features::default();
    let required_features: wgpu::Features =
        wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES;

    let adapter_features: wgpu::Features = wgpu::Features::default();
    let needed_limits = wgpu::Limits {
        max_storage_buffers_per_shader_stage: 8,
        max_storage_buffer_binding_size: 256 * 1024 * 1024,
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

    let size = window.inner_size();
    let surface_config = surface
        .get_default_config(&adapter, size.width, size.height)
        .expect("Surface isn't supported by the adapter.");
    surface.configure(&device, &surface_config);

    let app = App {
        instance,
        adapter,
        device,
        window,
        surface,
        surface_config,
        queue,
        executor: Spawner::new(),
        mouse: glam::Vec2::new(0.0, 0.0),
    };
    (event_loop, app)
}

pub trait Example: 'static + Sized {
    fn title() -> &'static str {
        "Example"
    }

    fn new(app: &App) -> Self;
    fn resize(&mut self, _: &App) {}
    fn event(&mut self, _: &App, _: winit::event::WindowEvent) {}
    fn update(&mut self, _: &App) {}
    fn render(&mut self, platform: &App, view: &wgpu::TextureView);
}

fn run<E: Example>(event_loop: EventLoop, app: App) {
    let mut example = E::new(&app);
    let mut app = app;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = if cfg!(feature = "metal-auto-capture") {
            ControlFlow::Exit
        } else {
            ControlFlow::Poll
        };
        match event {
            event::Event::RedrawEventsCleared => {
                #[cfg(not(target_arch = "wasm32"))]
                app.executor.run_until_stalled();
                app.window.request_redraw();
            }
            event::Event::WindowEvent {
                event:
                    WindowEvent::Resized(size)
                    | WindowEvent::ScaleFactorChanged {
                        new_inner_size: &mut size,
                        ..
                    },
                ..
            } => {
                app.surface_config.width = size.width.max(1);
                app.surface_config.height = size.height.max(1);
                example.resize(&app);
                app.surface.configure(&app.device, &app.surface_config);
            }
            event::Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                app.mouse = glam::Vec2::new(
                    position.x as f32 / app.surface_config.width as f32,
                    position.y as f32 / app.surface_config.height as f32,
                );
                app.mouse.y = 1.0 - app.mouse.y;
                app.mouse = app.mouse * 2.0 - 1.0
            }
            event::Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => example.event(&app, event),
            },
            event::Event::RedrawRequested(_) => {
                let frame = match app.surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(_) => {
                        app.surface.configure(&app.device, &app.surface_config);
                        app.surface
                            .get_current_texture()
                            .expect("Failed to acquire next surface texture!")
                    }
                };
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                example.update(&app);
                example.render(&app, &view);
                frame.present();
            }
            _ => {}
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
pub fn start<E: Example>() {
    let (event_loop, platform) = pollster::block_on(initialize::<E>(E::title()));
    run::<E>(event_loop, platform);
}

#[cfg(target_arch = "wasm32")]
pub fn start<E: Example>() {
    use wasm_bindgen::{prelude::*, JsCast};

    console_log::init_with_level(log::Level::Error).expect("could not initialize logger");
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    wasm_bindgen_futures::spawn_local(async move {
        let app = initialize::<Self>(Self::title()).await;
        let start_closure = Closure::once_into_js(move || self.run([app]));

        // make sure to handle JS exceptions thrown inside start.
        // Otherwise wasm_bindgen_futures Queue would break and never handle any tasks again.
        // This is required, because winit uses JS exception for control flow to escape from `run`.
        if let Err(error) = call_catch(&start_closure) {
            let is_control_flow_exception = error.dyn_ref::<js_sys::Error>().map_or(false, |e| {
                e.message().includes("Using exceptions for control flow", 0)
            });

            if !is_control_flow_exception {
                web_sys::console::error_1(&error);
            }
        }

        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(catch, js_namespace = Function, js_name = "prototype.call.call")]
            fn call_catch(this: &JsValue) -> Result<(), JsValue>;
        }
    });
}
