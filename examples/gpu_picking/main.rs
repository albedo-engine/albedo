#[path = "../example/mod.rs"]
mod example;
use albedo_backend::gpu::{self, AsVertexBufferLayout, ResourceBuilder};
use albedo_backend::mesh::{shapes, AsVertexFormat, ToPrimitive};
use albedo_bvh::{builders::SAHBuilder, BLASArray};
use albedo_rtx::{
    uniforms::{Ray, Vertex},
    Instance,
};

use nanorand::Rng;
use std::borrow::Cow;

use example::Example;

struct Camera {
    perspective: glam::Mat4,
}

impl Camera {
    pub fn new(aspect_ratio: f32) -> Self {
        Self {
            perspective: glam::Mat4::perspective_rh_gl(0.38, aspect_ratio, 0.01, 100.0),
        }
    }
}

struct PickingExample {
    camera: Camera,
    depth_view: wgpu::TextureView,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    primitive_gpu: gpu::Primitive,
    uniforms_data: Vec<Uniforms>,
    ray_buffer: gpu::Buffer<Ray>,
    scene_bindgroup: wgpu::BindGroup,
    intersection_pass_bg: wgpu::BindGroup,
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
        let depth_texture = app.device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: app.surface_config.width,
                height: app.surface_config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
            view_formats: &[],
        });

        let bgl = app
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Highlight Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let shader = app
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
            });

        let primitive = shapes::Cube::new(1.0)
            .to_primitive(Vertex::as_vertex_formats())
            .unwrap();

        let vertex_buffer_layout = primitive.as_vertex_buffer_layout();

        let pipeline_layout = app
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bgl],
                push_constant_ranges: &[],
            });

        let pipeline = app
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[vertex_buffer_layout.build(None)],
                },
                label: None,
                layout: Some(&pipeline_layout),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(app.surface_config.format.into())],
                }),
                primitive: wgpu::PrimitiveState {
                    cull_mode: Some(wgpu::Face::Back),
                    ..Default::default()
                },
                multisample: wgpu::MultisampleState::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24Plus,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multiview: None,
            });

        let primitive_gpu = albedo_backend::mesh::PrimitiveResourceBuilder::new(&primitive)
            .descriptor(gpu::BufferInitDescriptor::new(
                Some("Primtive Buffers"),
                wgpu::BufferUsages::VERTEX,
            ))
            .build(&app.device)
            .unwrap();

        let camera =
            Camera::new(app.surface_config.width as f32 / app.surface_config.height as f32);

        const NB_INSTANCES: usize = 100;
        let mut instances: Vec<Instance> = Vec::with_capacity(NB_INSTANCES);
        let mut rng = nanorand::WyRand::new_seed(42);
        let mut rand_val = |len: f32| rng.generate::<f32>() * len - 0.5 * len;
        let mut uniforms_data: Vec<Uniforms> = Vec::with_capacity(NB_INSTANCES);
        for _ in 0..NB_INSTANCES {
            let local_to_world = glam::Mat4::from_translation(glam::Vec3::new(
                rand_val(20.0),
                rand_val(20.0),
                rand_val(10.0) - 40.0,
            ));
            instances.push(Instance::from_transform(local_to_world.clone()));
            uniforms_data.push(Uniforms {
                transform: camera.perspective * local_to_world,
                color: glam::Vec4::new(rand_val(1.0), rand_val(1.0), rand_val(1.0), 1.0),
            });
        }

        let ray_buffer = gpu::Buffer::new_storage(&app.device, 1, None);
        let intersection_buffer = gpu::Buffer::new_storage(&app.device, 1, None);
        let uniform_buffer = gpu::Buffer::new_storage_with_data(&app.device, &uniforms_data, None);

        let bind_group = app.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: intersection_buffer.as_entire_binding(),
                },
            ],
            label: Some("Bind Group"),
        });

        let scene_bgl = albedo_rtx::RTGeometryBindGroupLayout::new(&app.device);

        let mut builder = SAHBuilder::new();
        let blas = BLASArray::new(std::slice::from_ref(&primitive), &mut builder).unwrap();

        let instance_buffer = gpu::Buffer::new_storage_with_data(&app.device, &instances, None);
        let bvh_buffer = gpu::Buffer::new_storage_with_data(&app.device, &blas.nodes, None);
        let index_buffer = gpu::Buffer::new_storage_with_data(
            &app.device,
            &blas.indices,
            Some(gpu::BufferInitDescriptor {
                label: None,
                usage: wgpu::BufferUsages::INDEX,
            }),
        );
        let vertex_buffer = gpu::Buffer::new_storage_with_data(
            &app.device,
            primitive.cast::<Vertex>().unwrap(),
            None,
        );
        let light_buffer = gpu::Buffer::dummy_storage(&app.device);

        let intersection_pass =
            albedo_rtx::passes::IntersectorPass::new(&app.device, &scene_bgl, None);
        let intersection_pass_bg = intersection_pass.create_frame_bind_groups(
            &app.device,
            (1, 1),
            &intersection_buffer,
            &ray_buffer,
        );

        let scene_bindgroup = scene_bgl.create_bindgroup(
            &app.device,
            bvh_buffer.as_storage_slice().unwrap(),
            instance_buffer.as_storage_slice().unwrap(),
            index_buffer.as_storage_slice().unwrap(),
            vertex_buffer.as_storage_slice().unwrap(),
            light_buffer.as_storage_slice().unwrap(),
        );

        PickingExample {
            camera,

            depth_view: depth_texture.create_view(&wgpu::TextureViewDescriptor::default()),
            pipeline,
            bind_group,
            primitive_gpu,
            uniforms_data,

            ray_buffer,
            intersection_pass,
            intersection_pass_bg,
            scene_bindgroup,
        }
    }

    fn render(&mut self, app: &example::App, view: &wgpu::TextureView) {
        let mut encoder = app
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let mouse = glam::Vec3::new(app.mouse.x, app.mouse.y, 0.5);
        let dir = self
            .camera
            .perspective
            .inverse()
            .project_point3(mouse)
            .normalize();

        let ray = Ray::from_origin_dir(&glam::Vec3::ZERO, dir);
        self.ray_buffer.update(&app.queue, &[ray]);

        self.intersection_pass.dispatch(
            &mut encoder,
            &self.scene_bindgroup,
            &self.intersection_pass_bg,
            (1, 1, 1),
        );

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
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_index_buffer(
                self.primitive_gpu.indices.inner().slice(..),
                wgpu::IndexFormat::Uint16,
            );
            // @todo: Will be the same for all primitives.
            for i in 0..self.primitive_gpu.attributes.len() {
                rpass.set_vertex_buffer(
                    i as u32,
                    self.primitive_gpu.attributes[i].inner().slice(..),
                );
            }
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
