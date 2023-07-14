#[path = "../example/mod.rs"]
mod example;
use albedo_rtx::uniforms;
use example::Example;

use nanorand::Rng;

use std::borrow::Cow;

use albedo_backend::gpu::AsVertexBufferLayout;
use albedo_backend::gpu::{self, ResourceBuilder};

use albedo_backend::mesh::shapes::Shape;
use albedo_backend::mesh::*;

struct SceneGpu {
    pub instance_buffer: gpu::Buffer<albedo_rtx::Instance>,
    pub bvh_buffer: gpu::Buffer<albedo_bvh::BVHNode>,
    pub index_buffer: gpu::Buffer<u32>,
    pub vertex_buffer: gpu::Buffer<albedo_rtx::Vertex>,
    pub light_buffer: gpu::Buffer<albedo_rtx::Light>,
}

struct PickingExample {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    primitive_gpu: gpu::Primitive,
    uniforms_data: Vec<Uniforms>,
    scene_bgl: albedo_rtx::RTSceneBindGroupLayout,
    // scene_gpu: SceneGpu,
    // intersection_pass_bg: wgpu::BindGroup,
    intersection_pass: albedo_rtx::passes::IntersectorPass,
}

// @todo: Create a UniformBlock derive
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Uniforms {
    transform: glam::Mat4,
    color: glam::Vec4,
}
unsafe impl bytemuck::Pod for Uniforms {}
unsafe impl bytemuck::Zeroable for Uniforms {}

impl Example for PickingExample {
    fn new(app: &example::App) -> Self {
        let bgl = gpu::BindGroupLayoutBuilder::new_with_size(1)
            .storage_buffer(
                wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                true,
            )
            .build(&app.device);

        let shader = app
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
            });

        let primitive = shapes::Cube::new(1.0)
            .data()
            .to_primitive(uniforms::Vertex::as_vertex_formats())
            .unwrap();

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

        let primitive_gpu = PrimitiveResourceBuilder::new(&primitive)
            .descriptor(gpu::BufferInitDescriptor::new(
                Some("Primtive Buffers"),
                wgpu::BufferUsages::VERTEX,
            ))
            .build(&app.device)
            .unwrap();

        let aspect_ratio = app.surface_config.width as f32 / app.surface_config.height as f32;

        let cam_transform = glam::Mat4::perspective_rh_gl(0.38, aspect_ratio, 0.01, 100.0);
        const NB_INSTANCES: usize = 100;
        let mut rng = nanorand::WyRand::new_seed(42);
        let mut rand_val = |len: f32| rng.generate::<f32>() * len - 0.5 * len;
        let mut uniforms_data: Vec<Uniforms> = Vec::with_capacity(NB_INSTANCES);
        for _ in 0..NB_INSTANCES {
            let local_to_world = glam::Mat4::from_translation(glam::Vec3::new(
                rand_val(20.0),
                rand_val(20.0),
                rand_val(10.0) - 40.0,
            ));
            uniforms_data.push(Uniforms {
                transform: cam_transform * local_to_world,
                color: glam::Vec4::new(rand_val(1.0), rand_val(1.0), rand_val(1.0), 1.0),
            });
        }

        let uniform_buffer = gpu::Buffer::new_storage_with_data(&app.device, &uniforms_data, None);

        let bind_group = app.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("Bind Group"),
        });

        let scene_bgl = albedo_rtx::RTSceneBindGroupLayout::new(&app.device);

        // Create scene containing bvh, vertices, etc...
        // let instances = Vec::with_capacity(NB_INSTANCES);

        PickingExample {
            pipeline,
            bind_group,
            primitive_gpu,
            uniforms_data,

            // scene_gpu: SceneGpu {
            //     instance_buffer:
            //     bvh_buffer:
            //     index_buffer:
            //     vertex_buffer:
            //     light_buffer:
            // },
            intersection_pass: albedo_rtx::passes::IntersectorPass::new(
                &app.device,
                &scene_bgl,
                None,
            ),
            scene_bgl,
        }
    }

    fn resize(&mut self, _app: &example::App) {}
    fn event(&mut self, _: &example::App, _: winit::event::WindowEvent) {}
    fn update(&mut self, _: &example::App) {}

    fn render(&mut self, app: &example::App, view: &wgpu::TextureView) {
        let mut encoder = app
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // self.intersection_pass
        //     .dispatch(&mut encoder, &self.intersection_pass_bg, (1, 1, 1));

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
                self.primitive_gpu.indices.inner().slice(..),
                wgpu::IndexFormat::Uint16,
            );
            for i in 0..self.primitive_gpu.attributes.len() {
                rpass.set_vertex_buffer(
                    i as u32,
                    self.primitive_gpu.attributes[i].inner().slice(..),
                );
            }
            rpass.pop_debug_group();
            rpass.insert_debug_marker("Draw!");
            rpass.draw_indexed(
                0..self.primitive_gpu.indices.count() as u32,
                0,
                0..(self.uniforms_data.len() as u32),
            );
        }

        app.queue.submit(Some(encoder.finish()));
    }
}

fn main() {
    example::start::<PickingExample>();
}
