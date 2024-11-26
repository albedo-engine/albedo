use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::Deref;

use crate::get_dispatch_size;
use crate::macros::path_separator;
use crate::DenoiseResources;
use crate::RTGeometryBindGroupLayout;
use crate::RTSurfaceBindGroupLayout;
use crate::RaytraceResources;
use albedo_backend::data::CompileError;
use albedo_backend::data::PreprocessError;
use albedo_backend::data::ShaderCache;
use bitflags::bitflags;
use wgpu::naga::FastHashMap;
use wgpu::PushConstantRange;

use super::GBUFFER_WRITE_TY;

bitflags! {
    pub struct ShadingFlags: u32 {
        const EMIT_GBUFFER = 0b00000001;
    }
}

pub struct ShadingBindGroupLayout {
    inner: wgpu::BindGroupLayout,
    flags: ShadingFlags
}

impl ShadingBindGroupLayout {
    const RAY_BINDING: u32 = 0;
    const INTERSECTION_BINDING: u32 = 1;
    const PER_DRAW_STRUCT_BINDING: u32 = 2;
    const GBUFFER_BINDING: u32 = 3;
    const MOTION_BINDING: u32 = 4;

    pub fn new(device: &wgpu::Device, defines: &FastHashMap<String, String>) -> Self {
        let flags = {
            let mut f = ShadingFlags::empty();
            if defines.contains_key("EMIT_GBUFFER") {
                f = f | ShadingFlags::EMIT_GBUFFER;
            }
            f
        };

        let mut entries: Vec<wgpu::BindGroupLayoutEntry> = Vec::new();
        entries.extend_from_slice(&[
            wgpu::BindGroupLayoutEntry {
                binding: Self::RAY_BINDING,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: Self::INTERSECTION_BINDING,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: Self::PER_DRAW_STRUCT_BINDING,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ]);

        if flags.contains(ShadingFlags::EMIT_GBUFFER) {
            entries.extend_from_slice(&[
                wgpu::BindGroupLayoutEntry {
                    binding: Self::GBUFFER_BINDING,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: GBUFFER_WRITE_TY,
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: Self::MOTION_BINDING,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        format: wgpu::TextureFormat::Rg32Float,
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ]);
        }

        Self {
            inner: device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Shading View Bind Group Layout"),
                entries: &entries,
            }),
            flags
        }
    }

    pub fn as_bind_group(&self, device: &wgpu::Device, resources: &RaytraceResources, denoise: Option<&DenoiseResources>) -> wgpu::BindGroup {
        let mut entries: Vec<wgpu::BindGroupEntry<'_>> = Vec::new();

        entries.extend_from_slice(&[
            wgpu::BindGroupEntry {
                binding: Self::RAY_BINDING,
                resource: resources.rays.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: Self::INTERSECTION_BINDING,
                resource: resources.intersections.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: Self::PER_DRAW_STRUCT_BINDING,
                resource: resources.global_uniforms.as_entire_binding(),
            },
        ]);

        if self.flags.contains(ShadingFlags::EMIT_GBUFFER) {
            let Some(denoise) = denoise else {
                panic!("Oopsie")
            };
            entries.push(wgpu::BindGroupEntry {
                binding: Self::GBUFFER_BINDING,
                resource: wgpu::BindingResource::TextureView(denoise.gbuffer_current),
            });
            entries.push(wgpu::BindGroupEntry {
                binding: Self::MOTION_BINDING,
                resource: wgpu::BindingResource::TextureView( denoise.motion),
            });
        }

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Radiance Estimator Frame Bind Group"),
            layout: &self.inner,
            entries: &entries
        })
    }
}

pub struct ShadingPass {
    pub bgl: ShadingBindGroupLayout,
    pipeline: wgpu::ComputePipeline,
}

impl ShadingPass {
    const WORKGROUP_SIZE: (u32, u32, u32) = (8, 8, 1);
    const SHADER_ID: &'static str = "shading.comp";

    pub fn new(
        device: &wgpu::Device,
        processor: &ShaderCache,
        defines: &FastHashMap<String, String>,
        geometry_layout: &RTGeometryBindGroupLayout,
        surface_layout: &RTSurfaceBindGroupLayout,
    ) -> Result<Self, CompileError> {
        let Some(source) = processor.get(Self::SHADER_ID) else {
            return Err(PreprocessError::Missing(Self::SHADER_ID.to_string()).into());
        };
        Self::new_raw(device, geometry_layout, surface_layout, processor, defines, source)
    }

    pub fn new_inlined(
        device: &wgpu::Device,
        processor: &ShaderCache,
        defines: &FastHashMap<String, String>,
        geometry_layout: &RTGeometryBindGroupLayout,
        surface_layout: &RTSurfaceBindGroupLayout,
    ) -> Self {
        Self::new_raw(device, geometry_layout, surface_layout, processor, defines, include_str!(concat!(
            "..",
            path_separator!(),
            "..",
            path_separator!(),
            "shaders",
            path_separator!(),
            "shading.comp"
        ))).unwrap()
    }

    pub fn dispatch(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        geometry_bindgroup: &wgpu::BindGroup,
        surface_bindgroup: &wgpu::BindGroup,
        frame_bind_groups: &wgpu::BindGroup,
        size: (u32, u32, u32),
    ) {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Shading Pass"),
            timestamp_writes: None,
        });
        let workgroups = get_dispatch_size(&size, &Self::WORKGROUP_SIZE);
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, geometry_bindgroup, &[]);
        pass.set_bind_group(1, surface_bindgroup, &[]);
        pass.set_bind_group(2, frame_bind_groups, &[]);
        pass.dispatch_workgroups(workgroups.0, workgroups.1, workgroups.2);
    }

    pub fn new_raw(
        device: &wgpu::Device,
        geometry_layout: &RTGeometryBindGroupLayout,
        surface_layout: &RTSurfaceBindGroupLayout,
        processor: &ShaderCache,
        defines: &FastHashMap<String, String>,
        source: &str
    ) -> Result<Self, CompileError> {
        let bgl: ShadingBindGroupLayout = ShadingBindGroupLayout::new(device, defines);

        let push_constants = [wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::COMPUTE,
                range: 0..64,
        }];
        let push_constant_ranges = if bgl.flags.contains(ShadingFlags::EMIT_GBUFFER) {
            &push_constants
        } else {
            &[] as &[PushConstantRange]
        };

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Shading Pipeline Layout"),
            bind_group_layouts: &[geometry_layout, surface_layout, &bgl.inner],
            push_constant_ranges,
        });

        let module = processor.compile_compute(source, Some(defines))?;
        let shader: wgpu::ShaderModule = device.create_shader_module(wgpu::ShaderModuleDescriptor{
            label: Some(Self::SHADER_ID),
            source: wgpu::ShaderSource::Naga(Cow::Owned(module))
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Shading Pipeline"),
            layout: Some(&layout),
            entry_point: Some("main"),
            module: &shader,
            compilation_options: Default::default(),
            cache: None,
        });

        Ok(Self {
            bgl,
            pipeline,
        })
    }
}

pub struct PrimaryRayPass(ShadingPass);

impl PrimaryRayPass {
    pub fn new_inlined(
        device: &wgpu::Device,
        processor: &ShaderCache,
        geometry_layout: &RTGeometryBindGroupLayout,
        surface_layout: &RTSurfaceBindGroupLayout,
    ) -> Self {
        // @todo: Replace by shader flags
        let mut defines: FastHashMap<String, String> = FastHashMap::default();
        defines.insert("EMIT_GBUFFER".into(), "".into());
        Self {
            0: ShadingPass::new_inlined(device, processor, &defines, geometry_layout, surface_layout)
        }
    }

    pub fn dispatch(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        geometry_bindgroup: &wgpu::BindGroup,
        surface_bindgroup: &wgpu::BindGroup,
        frame_bind_groups: &wgpu::BindGroup,
        size: (u32, u32, u32),
        world_to_screen: &glam::Mat4 // @todo: Better to not use GLAM probably here
    ) {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Primary Shading Pass"),
            timestamp_writes: None,
        });
        let workgroups = get_dispatch_size(&size, &ShadingPass::WORKGROUP_SIZE);
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, geometry_bindgroup, &[]);
        pass.set_bind_group(1, surface_bindgroup, &[]);
        pass.set_bind_group(2, frame_bind_groups, &[]);
        {
            let data: &[f32; 16] = world_to_screen.as_ref();
            let data = bytemuck::cast_slice(data);
            pass.set_push_constants(0, data);
        }
        pass.dispatch_workgroups(workgroups.0, workgroups.1, workgroups.2);
    }
}

impl Deref for PrimaryRayPass {
    type Target = ShadingPass;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
