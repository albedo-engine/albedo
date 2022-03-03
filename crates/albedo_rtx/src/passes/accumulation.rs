use crate::renderer::resources;
use albedo_backend::{
    shader_bindings,
    GPUBuffer, UniformBuffer,
    ComputePassDescription
};

pub struct AccumulationPassDescription {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
}

impl AccumulationPassDescription {
    pub fn new(device: &wgpu::Device) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Ray Generator Layout"),
            entries: &[
                shader_bindings::buffer_entry(0, wgpu::ShaderStages::COMPUTE, true),
                shader_bindings::storage_texture2d_entry(1, wgpu::ShaderStages::COMPUTE, wgpu::TextureFormat::Rgba32Float, wgpu::StorageTextureAccess::ReadWrite),
                shader_bindings::uniform_entry(2, wgpu::ShaderStages::COMPUTE),
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Accumulation Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader =
            device.create_shader_module(&wgpu::include_spirv!("../shaders/accumulation.comp.spv"));

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Accumulation Pipeline"),
            layout: Some(&pipeline_layout),
            entry_point: "main",
            module: &shader,
        });

        AccumulationPassDescription {
            bind_group_layout,
            pipeline,
        }
    }

    pub fn create_frame_bind_groups(
        &self,
        device: &wgpu::Device,
        in_rays: &GPUBuffer<resources::RayGPU>,
        view: &wgpu::TextureView,
        global_uniforms: &UniformBuffer<resources::GlobalUniformsGPU>,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Accumulation Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: in_rays.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: global_uniforms.as_entire_binding(),
                },
            ],
        })
    }
}

impl ComputePassDescription for AccumulationPassDescription {
    type FrameBindGroups = wgpu::BindGroup;
    type PassBindGroups = ();

    fn get_name() -> &'static str {
        "Accumulation Pass"
    }

    fn set_frame_bind_groups(pass: &mut wgpu::ComputePass, groups: &Self::FrameBindGroups) {
        pass.set_bind_group(0, groups, &[]);
    }

    fn set_pass_bind_groups(pass: &mut wgpu::ComputePass, groups: &Self::PassBindGroups) {
        todo!()
    }

    fn get_workgroup_size(&self) -> (u32, u32, u32) {
        (8, 8, 1)
    }

    fn get_pipeline(&self) -> &wgpu::ComputePipeline {
        &self.pipeline
    }
}

// impl<'a> AccumulationPass<'a> {
//     pub fn new(device: &wgpu::Device) -> Self {
//         let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
//             label: Some("Ray Generator Layout"),
//             entries: &[
//                 shader_bindings::buffer_entry(0, wgpu::ShaderStages::COMPUTE, true),
//                 shader_bindings::storage_texture2d_entry(1, wgpu::ShaderStages::COMPUTE, wgpu::TextureFormat::Rgba32Float, wgpu::StorageTextureAccess::ReadWrite),
//                 shader_bindings::uniform_entry(2, wgpu::ShaderStages::COMPUTE),
//             ],
//         });

//         let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
//             label: Some("Accumulation Pipeline Layout"),
//             bind_group_layouts: &[&bind_group_layout],
//             push_constant_ranges: &[],
//         });

//         let shader =
//             device.create_shader_module(&wgpu::include_spirv!("../shaders/accumulation.comp.spv"));

//         let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
//             label: Some("Accumulation Pipeline"),
//             layout: Some(&pipeline_layout),
//             entry_point: "main",
//             module: &shader,
//         });

//         AccumulationPass {
//             bind_group_layout,
//             pipeline,
//             inner_pass: None
//         }
//     }

//     pub fn create_bind_groups(
//         &self,
//         device: &wgpu::Device,
//         in_rays: &GPUBuffer<resources::RayGPU>,
//         view: &wgpu::TextureView,
//         global_uniforms: &UniformBuffer<resources::GlobalUniformsGPU>,
//     ) -> [wgpu::BindGroup; 1] {
//         [
//             device.create_bind_group(&wgpu::BindGroupDescriptor {
//                 label: Some("Accumulation Bind Group"),
//                 layout: &self.bind_group_layout,
//                 entries: &[
//                     wgpu::BindGroupEntry {
//                         binding: 0,
//                         resource: in_rays.as_entire_binding(),
//                     },
//                     wgpu::BindGroupEntry {
//                         binding: 1,
//                         resource: wgpu::BindingResource::TextureView(view),
//                     },
//                     wgpu::BindGroupEntry {
//                         binding: 2,
//                         resource: global_uniforms.as_entire_binding(),
//                     },
//                 ],
//             })
//         ]
//     }

// }
