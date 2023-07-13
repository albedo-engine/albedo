#[path = "../example/mod.rs"]
mod example;
use example::Example;

use nanorand::Rng;

use std::borrow::Cow;

use albedo_backend::gpu::AsVertexBufferLayout;
use albedo_backend::gpu::{self, ResourceBuilder};

use albedo_backend::mesh::shapes::Shape;
use albedo_backend::mesh::*;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex3D {
    position: [f32; 4],
    normal: [f32; 4],
}

unsafe impl bytemuck::Pod for Vertex3D {}
unsafe impl bytemuck::Zeroable for Vertex3D {}

impl AsVertexFormat for Vertex3D {
    fn as_vertex_formats() -> &'static [AttributeDescriptor] {
        static ATTRIBUTE_DESCRIPTORS: [AttributeDescriptor; 2] = [
            AttributeDescriptor {
                id: AttributeId::POSITION,
                format: wgpu::VertexFormat::Float32x4,
            },
            AttributeDescriptor {
                id: AttributeId::NORMAL,
                format: wgpu::VertexFormat::Float32x4,
            },
        ];
        &ATTRIBUTE_DESCRIPTORS
    }
}

struct PickingExample {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    primitive_gpu: gpu::Primitive,
    uniforms_data: Vec<Uniforms>,
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
            .storage_buffer(wgpu::ShaderStages::VERTEX, true)
            .build(&app.device);

        let shader = app
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
            });

        let cube_data = shapes::Cube::new(1.0).data();
        let mut primitive =
            Primitive::interleaved_with_count(cube_data.count(), Vertex3D::as_vertex_formats());
        primitive.set_indices(cube_data.indices);
        let mut positions = primitive.attribute::<[f32; 4]>(0).unwrap();
        positions.set(&cube_data.positions);

        let mut normals = primitive.attribute::<[f32; 4]>(1).unwrap();
        normals.set(&cube_data.normals);

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
                // wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
                wgpu::BufferUsages::VERTEX,
            ))
            .build(&app.device)
            .unwrap();

        let aspect_ratio = app.surface_config.width as f32 / app.surface_config.height as f32;

        let cam_transform = glam::Mat4::perspective_rh_gl(0.38, aspect_ratio, 0.01, 100.0);
        const NB_INSTANCES: usize = 100;
        let mut rng = nanorand::WyRand::new_seed(42);
        let mut rand_val = || rng.generate::<f32>() * 10.0 - 5.0;
        let mut uniforms_data: Vec<Uniforms> = Vec::with_capacity(NB_INSTANCES);
        for i in 0..NB_INSTANCES {
            let local_to_world = glam::Mat4::from_translation(glam::Vec3::new(
                rand_val(),
                rand_val(),
                rand_val() - 10.0,
            ));
            uniforms_data.push(Uniforms {
                transform: cam_transform * local_to_world,
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
        PickingExample {
            pipeline,
            bind_group,
            primitive_gpu,
            uniforms_data,
        }
    }

    fn resize(&mut self, _app: &example::App) {}
    fn update(&mut self, _event: winit::event::WindowEvent) {}

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
