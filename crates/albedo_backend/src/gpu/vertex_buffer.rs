pub struct VertexBufferLayoutBuilder {
    attributes: Vec<wgpu::VertexAttribute>,
}

impl VertexBufferLayoutBuilder {
    pub fn new(size: usize) -> Self {
        Self {
            attributes: Vec::with_capacity(size),
        }
    }

    pub fn build(&self, stride: Option<u64>) -> wgpu::VertexBufferLayout {
        let array_stride = if let Some(stride) = stride {
            stride
        } else {
            self.attributes
                .iter()
                .fold(0, |sum, v| sum + v.format.size())
        };
        wgpu::VertexBufferLayout {
            array_stride,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &self.attributes.as_ref(),
        }
    }

    pub fn auto_attribute(self, format: wgpu::VertexFormat) -> Self {
        let location = self.attributes.len() as u32;
        let offset = if let Some(last) = self.attributes.last() {
            last.offset + last.format.size()
        } else {
            0
        };
        self.attribute(format, location, offset)
    }

    pub fn attribute(
        mut self,
        format: wgpu::VertexFormat,
        shader_location: wgpu::ShaderLocation,
        offset: u64,
    ) -> Self {
        self.attributes.push(wgpu::VertexAttribute {
            format,
            shader_location,
            offset,
        });
        self
    }
}
