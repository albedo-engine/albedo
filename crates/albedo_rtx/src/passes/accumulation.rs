use crate::renderer::resources;
use albedo_backend::{shader_bindings, GPUBuffer, UniformBuffer};

pub struct AccumulationPass {
}

impl AccumulationPass {
    pub fn new(device: &wgpu::Device) -> Self {
        AccumulationPass {
        }
    }

    pub fn bind(
        &mut self,
    ) {
    }

    pub fn run(&self, encoder: &mut wgpu::CommandEncoder, width: u32, height: u32) {
    }
}
