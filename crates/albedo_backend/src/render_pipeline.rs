pub struct RenderPipelineBuilder<'a> {
    desc: wgpu::RenderPipelineDescriptor<'a>,
}

impl<'a> RenderPipelineBuilder<'a> {
    pub fn new(vertex: wgpu::VertexState<'a>) -> Self {
        RenderPipelineBuilder {
            desc: wgpu::RenderPipelineDescriptor {
                vertex,
                label: None,
                layout: None,
                fragment: None,
                primitive: wgpu::PrimitiveState {
                    cull_mode: Some(wgpu::Face::Back),
                    ..Default::default()
                },
                multisample: wgpu::MultisampleState::default(),
                depth_stencil: None,
                multiview: None,
            },
        }
    }

    pub fn build(self, device: &wgpu::Device) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&self.desc)
    }

    pub fn label(mut self, label: wgpu::Label<'a>) -> Self {
        self.desc.label = label;
        self
    }

    pub fn layout(mut self, layout: &'a wgpu::PipelineLayout) -> Self {
        self.desc.layout = Some(layout);
        self
    }

    pub fn fragment(mut self, state: Option<wgpu::FragmentState<'a>>) -> Self {
        self.desc.fragment = state;
        self
    }

    pub fn primitive(mut self, state: wgpu::PrimitiveState) -> Self {
        self.desc.primitive = state;
        self
    }
}
