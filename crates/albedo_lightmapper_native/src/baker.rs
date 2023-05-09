use albedo_backend::{gpu, mesh::IndexData};
use albedo_rtx::uniforms;

/// GPU data for a single mesh.
struct MeshData {
    index_buffer: gpu::IndexBuffer,
    vertex_buffer: gpu::Buffer<uniforms::Vertex>,
}

impl MeshData {
    pub fn vertices(&self) -> &gpu::Buffer<uniforms::Vertex> {
        &self.vertex_buffer
    }
    pub fn indices(&self) -> &gpu::IndexBuffer {
        &self.index_buffer
    }
}

pub struct Baker {
    data: Option<MeshData>,
}

impl Baker {
    pub fn new() -> Self {
        Baker { data: None }
    }

    pub fn set_mesh_data(
        &mut self,
        device: &wgpu::Device,
        vertices: &[uniforms::Vertex],
        indices: &[u32],
    ) {
        self.data = Some(MeshData {
            vertex_buffer: gpu::Buffer::new_vertex_with_data(device, vertices, None),
            index_buffer: gpu::IndexBuffer::new_with_data_32(device, indices, None),
        });
    }

    pub fn bake_into(device: &wgpu::Device, out: &mut crate::ImageSlice) -> Vec<f32> {
        vec![]
    }
}
