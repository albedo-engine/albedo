use std::{borrow::Cow, collections::HashMap};

use wgpu::naga::FastHashMap;

use crate::data::{CompileError, PreprocessError, ShaderCache};

pub trait ComputePipeline {
    const LABEL: &'static str;
    const SHADER_ID: &'static str;

    fn compile(
        device: &wgpu::Device,
        processor: &ShaderCache,
        layout: &wgpu::PipelineLayout,
        source: &str,
    ) -> Result<wgpu::ComputePipeline, CompileError> {
        let module = processor.compile_compute(source, None)?;

        let shader: wgpu::ShaderModule =
            device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(Self::SHADER_ID),
                source: wgpu::ShaderSource::Naga(Cow::Owned(module)),
            });

        Ok(
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(Self::LABEL),
                layout: Some(layout),
                entry_point: Some("main"),
                module: &shader,
                compilation_options: Default::default(),
                cache: None,
            }),
        )
    }

    fn recompile(
        &mut self,
        device: &wgpu::Device,
        processor: &ShaderCache,
    ) -> Result<(), CompileError> {
        let Some(source) = processor.get(Self::SHADER_ID) else {
            return Err(PreprocessError::Missing(Self::SHADER_ID.to_string()).into());
        };
        let pipeline = Self::compile(device, processor, self.get_pipeline_layout(), source)?;
        self.set_pipeline(pipeline);
        Ok(())
    }
    fn set_pipeline(&mut self, pipeline: wgpu::ComputePipeline);
    fn get_pipeline_layout(&self) -> &wgpu::PipelineLayout;
}

pub trait AsBindGroup<'a> {
    type Params;

    fn as_bind_group(
        &self,
        device: &wgpu::Device,
        defines: &HashMap<String, String>,
        params: &Self::Params,
    ) -> Result<wgpu::BindGroup, String>;
}
