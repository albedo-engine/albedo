use albedo_backend::gpu;
use albedo_bvh;
use albedo_bvh::{builders, BLASArray, Mesh};
use albedo_bvh::{FlatNode};
use albedo_rtx::uniforms::{Vertex};
use albedo_rtx::passes;
use albedo_rtx::uniforms::{Camera, Intersection, PerDrawUniforms, Ray, Uniform};
use albedo_rtx::uniforms::{Instance, Light, Material};
use std::sync::Mutex;
use std::convert::TryInto;

mod app;
pub use app::*;

#[repr(C)]
pub struct ImageSlice<'a> {
    width: u32,
    height: u32,
    data: &'a mut [f32],
}

impl<'a> ImageSlice<'a> {
    pub fn data(&self) -> &[f32] {
        &self.data
    }
    pub fn data_mut(&mut self) -> &mut [f32] {
        &mut self.data
    }
}

#[repr(C)]
pub struct MeshDescriptor {
    positions: *const f32,
    normals: *const f32,
    uvs: *const f32,
    indices: *const u32,
    vertex_count: u32,
    index_count: u32,
}

pub struct MeshData<'a> {
    pub positions: &'a [[f32; 3]],
    pub normals: &'a [[f32; 3]],
    pub uvs: Option<&'a [[f32; 2]]>,
    pub indices: &'a [u32],
    pub vertex_count: u32,
    pub index_count: u32,
}

impl<'a> Mesh<Vertex> for MeshData<'a> {
    fn index(&self, index: u32) -> Option<&u32> {
        self.indices.get(index as usize)
    }

    fn vertex(&self, index: u32) -> Vertex {
        let i = index as usize;
        let pos = self.positions[i];
        let normal = self.normals[i];
        let uv = match &self.uvs {
            Some(u) => u[i],
            None => [0.0, 0.0],
        };
        Vertex::new(&pos, &normal, Some(&uv))
    }

    fn vertex_count(&self) -> u32 {
        self.positions.len() as u32
    }

    fn index_count(&self) -> u32 {
        self.indices.len() as u32
    }

    fn position(&self, index: u32) -> Option<&[f32; 3]> {
        self.positions.get(index as usize)
    }
}

struct BindGroups {
    blit_pass: wgpu::BindGroup,
    lightmap_pass: wgpu::BindGroup,
}

impl BindGroups {
    fn new(
        context: &GpuContext,
        render_target_view: &wgpu::TextureView,
        scene_resources: &SceneGPU,
        //probe: Option<&ProbeGPU>,
        global_uniforms: &gpu::Buffer<PerDrawUniforms>,
        camera_uniforms: &gpu::Buffer<Camera>,
        //ray_pass_desc: &passes::RayPass,
        //intersector_pass_desc: &passes::IntersectorPass,
        //shading_pass_desc: &passes::ShadingPass,
        //accumulation_pass_desc: &passes::AccumulationPass,
        blit_pass: &passes::BlitPass,
        lightmap_pass: &passes::LightmapPass,
    ) -> Self {
        //let texture_info_view = match &scene_resources.atlas {
        //    Some(atlas) => atlas.info_texture_view(),
        //    _ => device.default_textures().non_filterable_1d(),
        //};
        //let atlas_view = match &scene_resources.atlas {
        //    Some(atlas) => atlas.texture_view(),
        //    _ => device.default_textures().filterable_2darray(),
        //};
        //let probe = match probe {
        //    Some(p) => &p.view,
        //    _ => device.default_textures().filterable_2d(),
        //};
        BindGroups {
            blit_pass: blit_pass.create_frame_bind_groups(
                context.device(),
                &render_target_view,
                &context.sampler_nearest,
                global_uniforms,
            ),
            lightmap_pass: lightmap_pass.create_frame_bind_groups(
                context.device(),
                &scene_resources.instance_buffer,
                &scene_resources.bvh_buffer.inner(),
                &scene_resources.index_buffer,
                &scene_resources.vertex_buffer.inner(),
                global_uniforms
            ),
        }
    }
}

pub struct Passes {
    pub blit: passes::BlitPass,
    pub lightmap: passes::LightmapPass,
}

pub struct Renderer {
    render_target_view: wgpu::TextureView,

    camera: Camera,
    camera_uniforms: gpu::Buffer<Camera>,
    global_uniforms: PerDrawUniforms,
    global_uniforms_buffer: gpu::Buffer<PerDrawUniforms>,

    pub passes: Passes,
    bindgroups: Option<BindGroups>,

    size: (u32, u32),
}

#[derive(Debug)]
pub enum Error {
    FileNotFound(String),
    TextureToBufferReadFail,
    //ImageError(ImageError),
    AccelBuild(String),
}

impl From<Error> for String {
    fn from(e: Error) -> Self {
        match e {
            Error::FileNotFound(filename) => {
                format!("file not found: {}", filename)
            }
            //Error::ImageError(e) => {
            //    format!("file not found: {:?}", e)
            //}
            Error::TextureToBufferReadFail => String::from("failed to read pixels from GPU to CPU"),
            Error::AccelBuild(reason) => {
                format!("failed to build acceleration structure: {:?}", reason)
            }
        }
    }
}

/*impl From<ImageError> for Error {
    fn from(e: ImageError) -> Self {
        Error::ImageError(e)
    }
}*/

impl Renderer {
    pub fn max_ssbo_element_in_bytes() -> u32 {
        [
            Ray::size_in_bytes(),
            Intersection::size_in_bytes(),
            Camera::size_in_bytes(),
            PerDrawUniforms::size_in_bytes(),
        ]
        .iter()
        .fold(0, |max, &val| std::cmp::max(max, val))
    }

    pub fn create_target(context: &GpuContext, size: (u32, u32)) -> wgpu::Texture {
        let render_target = context.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("Main Render Target"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        render_target
    }

    pub fn new(context: &GpuContext, size: (u32, u32), swapchain_format: wgpu::TextureFormat) -> Self {
        let downsample_factor = 0.25;
        let downsampled_size = (
            (size.0 as f32 * downsample_factor) as u32,
            (size.1 as f32 * downsample_factor) as u32,
        );
        let render_target = Renderer::create_target(&context, size);
        Self {
            render_target_view: render_target.create_view(&wgpu::TextureViewDescriptor::default()),
            camera: Default::default(),
            camera_uniforms: gpu::Buffer::new_uniform(context.device(), 1, None),
            global_uniforms: PerDrawUniforms {
                frame_count: 1,
                seed: 0,
                ..Default::default()
            },
            global_uniforms_buffer: gpu::Buffer::new_uniform(context.device(), 1, None),
            passes: Passes {
                blit: passes::BlitPass::new(context.device(), swapchain_format),
                lightmap: passes::LightmapPass::new(context.device()),
            },
            bindgroups: None,
            size,
        }
    }

    /*pub fn update_camera(&mut self, origin: glam::Vec3, right: glam::Vec3, up: glam::Vec3) {
        self.camera.origin = origin;
        self.camera.right = right;
        self.camera.up = up;
    }*/

    pub fn resize(
        &mut self,
        context: &GpuContext,
        scene_resources: &SceneGPU,
        //probe: Option<&ProbeGPU>,
        size: (u32, u32),
    ) {
        self.size = size;
        self.render_target_view = Renderer::create_target(context, self.size).create_view(&wgpu::TextureViewDescriptor::default());
        self.set_resources(context, scene_resources);//, probe);
    }

    pub fn lightmap(&mut self, encoder: &mut wgpu::CommandEncoder, scene_resources: &SceneGPU) {
        let (bindgroups, render_target_view) = (self.bindgroups.as_ref().unwrap(), &self.render_target_view);

        self.passes
            .lightmap
            .draw(encoder,
                &render_target_view,
                &bindgroups.lightmap_pass,
                &scene_resources.instance_buffer,
                &scene_resources.index_buffer,
                &scene_resources.vertex_buffer.inner());

        self.global_uniforms.frame_count = 1;
    }

    pub fn blit(&mut self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let bindgroups = self.bindgroups.as_ref().unwrap();

        #[cfg(not(feature = "accumulate_read_write"))]
        let bindgroup = &bindgroups.blit_pass;
        #[cfg(feature = "accumulate_read_write")]
        let bindgroup = if self.global_uniforms.frame_count % 2 != 0 {
            &bindgroups.blit_pass
        } else {
            &bindgroups.blit_pass2
        };

        self.passes.blit.draw(encoder, &view, bindgroup);
    }

    pub fn reset_accumulation(&mut self) {
        self.global_uniforms.frame_count = 1;
        self.global_uniforms.seed = 0;
    }

    pub fn get_size(&self) -> (u32, u32) {
        self.size
    }

    pub fn set_resources(
        &mut self,
        context: &GpuContext,
        scene_resources: &SceneGPU,
        //probe: Option<&ProbeGPU>,
    ) {
        self.bindgroups = Some(BindGroups::new(
            context,
            &self.render_target_view,
            &scene_resources,
            //probe,
            &self.global_uniforms_buffer,
            &self.camera_uniforms,
            //&self.passes.rays,
            //&self.passes.intersection,
            //&self.passes.shading,
            //&self.passes.accumulation,
            &self.passes.blit,
            &self.passes.lightmap,
        ));
        self.global_uniforms.frame_count = 1;
    }

    pub async fn read_pixels(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<Vec<u8>, Error> {
        let alignment = albedo_backend::Alignment2D::texture_buffer_copy(
            self.size.0 as usize,
            std::mem::size_of::<u32>(),
        );
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Read Pixel Encoder"),
        });
        let (width, height) = self.size;
        let gpu_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: height as u64 * alignment.padded_bytes() as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let texture_extent = wgpu::Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            label: None,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // @todo: this re-create shaders + pipeline layout + life.
        let blit_pass = passes::BlitPass::new(device, wgpu::TextureFormat::Rgba8UnormSrgb);
        blit_pass.draw(
            &mut encoder,
            &view,
            &self.bindgroups.as_ref().unwrap().blit_pass,
        );

        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::ImageCopyBuffer {
                buffer: &gpu_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(
                        std::num::NonZeroU32::new(alignment.padded_bytes() as u32).unwrap(),
                    ),
                    rows_per_image: None,
                },
            },
            texture_extent,
        );
        queue.submit(Some(encoder.finish()));

        let buffer_slice = gpu_buffer.slice(..);
        // Sets the buffer up for mapping, sending over the result of the mapping back to us when it is finished.
        let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

        device.poll(wgpu::Maintain::Wait);

        if let Some(Ok(())) = receiver.receive().await {
            let padded_buffer = buffer_slice.get_mapped_range();
            let mut bytes: Vec<u8> = vec![0; alignment.unpadded_bytes_per_row * height as usize];
            // from the padded_buffer we write just the unpadded bytes into the image
            for (padded, bytes) in padded_buffer
                .chunks_exact(alignment.padded_bytes_per_row)
                .zip(bytes.chunks_exact_mut(alignment.unpadded_bytes_per_row))
            {
                bytes.copy_from_slice(&padded[..alignment.unpadded_bytes_per_row]);
            }
            // With the current interface, we have to make sure all mapped views are
            // dropped before we unmap the buffer.
            drop(padded_buffer);
            gpu_buffer.unmap();
            Ok(bytes)
        } else {
            Err(Error::TextureToBufferReadFail)
        }
    }
}

static app: Mutex<Option<App>> = Mutex::new(None);

#[no_mangle]
pub extern "C" fn init() {
    println!("Hello from Rust");
    unsafe {
        *app.lock().unwrap() = Some(App::new());
    }
}

#[no_mangle]
pub extern "C" fn set_mesh_data(desc: MeshDescriptor) {
    let count = desc.vertex_count / 3;
    if count % 3 != 0 {
        panic!("Vertex count must be a multiple of 3");
    }

    println!("Seting mesh data...");

    let mesh_data = MeshData {
        indices: unsafe { std::slice::from_raw_parts(desc.indices, desc.index_count as usize) },
        positions: unsafe { std::slice::from_raw_parts(desc.positions as *const [f32; 3], desc.vertex_count as usize) },
        normals: unsafe { std::slice::from_raw_parts(desc.normals as *const [f32; 3], desc.vertex_count as usize) },
        uvs: Some(unsafe { std::slice::from_raw_parts(desc.uvs as *const [f32; 2], desc.vertex_count as usize) }),
        index_count: desc.index_count,
        vertex_count: desc.vertex_count,
    };

    let indices = unsafe { std::slice::from_raw_parts(desc.indices, desc.index_count as usize) };

    // @todo: Skip conversion by making the BVH / GPU struct split the vertex.
    let mut vertices: Vec<Vertex> = Vec::with_capacity(count as usize);
    for i in 0..count as usize {
        vertices.push(Vertex::new(&mesh_data.positions[i], &mesh_data.normals[i], None));
    }

    let instance = Instance::default();

    let mut builder = builders::SAHBuilder::new();
    let result =
        BLASArray::new(&[mesh_data], &mut builder);

    let blas = match result {
        Ok(val) => val,
        Err(str) => return,
    };
    let mut guard = app.lock().unwrap();
    let runtime = guard.as_mut().unwrap();
    runtime.scene = Some(SceneGPU::new(
        runtime.context.device(),
        &[instance],
        &blas.nodes,
        indices,
        &vertices,
    ));
}

#[no_mangle]
pub extern "C" fn bake(raw_slice: *mut ImageSlice) {
    println!("Baking...");
    if raw_slice.is_null() {
        return;
    }
    let mut guard = app.lock().unwrap();
    let runtime = guard.as_mut().unwrap();
    let slice = unsafe { raw_slice.as_mut() }.unwrap();
    println!("Baking...2");

    let context = &runtime.context;

    println!("\n============================================================");
    println!("                   🚀 Albedo Pathtracer 🚀                   ");
    println!("============================================================\n");

    let init_size = (512, 512);

    let mut renderer = Renderer::new(
        &context,
        (init_size.0, init_size.1),
        wgpu::TextureFormat::Bgra8UnormSrgb,
    );

    let mut scene = match &runtime.scene {
        Some(val) => val,
        None => return,
    };

    //#[cfg(not(target_arch = "wasm32"))]
    //{
    //    app_context.load_env_path("./assets/uffizi-large.hdr");
    //    app_context
    //        .load_file_path("./assets/DamagedHelmet.glb")
    //        .unwrap();
    //}

    #[cfg(not(target_arch = "wasm32"))]
    let mut last_time = std::time::Instant::now();

    #[cfg(target_arch = "wasm32")]
    let win_performance = web_sys::window()
        .unwrap()
        .performance()
        .expect("performance should be available");
    #[cfg(target_arch = "wasm32")]
    let mut last_time = win_performance.now();

    // RENDER LOOP
    // Updates.
    #[cfg(not(target_arch = "wasm32"))]
    let (now, delta) = {
        let now = std::time::Instant::now();
        (now, now.duration_since(last_time).as_secs_f32())
    };
    #[cfg(target_arch = "wasm32")]
    let (now, delta) = {
        let now = win_performance.now();
        (now, ((now - last_time) / 1000.0) as f32)
    };
    last_time = now;

    //let (camera_right, camera_up) = camera_controller.update(delta);

    let mut encoder = context
        .device()
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    //renderer.update_camera(camera_controller.origin, camera_right, camera_up);
    //if !app_context.settings.accumulate || !camera_controller.is_static() {
    //    renderer.reset_accumulation();
    //}
    //renderer.raytrace(&mut encoder, &app_context.platform.queue);
    renderer.lightmap(&mut encoder, &scene);
    //renderer.blit(&mut encoder, &view);
    //renderer.accumulate = true;
}
