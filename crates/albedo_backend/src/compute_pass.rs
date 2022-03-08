pub trait ComputePassDescriptor {
    type FrameBindGroups: Sized;
    type PassBindGroups: Sized;

    fn get_name() -> &'static str;
    fn set_frame_bind_groups<'a, 'b>(
        pass: &mut wgpu::ComputePass<'a>,
        groups: &'b Self::FrameBindGroups,
    ) where
        'b: 'a;
    fn set_pass_bind_groups(pass: &mut wgpu::ComputePass, groups: &Self::PassBindGroups);
    fn get_pipeline(&self) -> &wgpu::ComputePipeline;
}

pub struct ComputePass<'a, Description: ComputePassDescriptor> {
    inner_pass: wgpu::ComputePass<'a>,
    _desc: std::marker::PhantomData<Description>,
}

impl<'a, 'b, Description> ComputePass<'a, Description>
where
    Description: ComputePassDescriptor,
    'b: 'a,
{
    pub fn new(
        encoder: &'a mut wgpu::CommandEncoder,
        description: &'b Description,
        frame_bind_groups: &'a Description::FrameBindGroups,
    ) -> ComputePass<'a, Description> {
        let mut inner_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some(Description::get_name()),
        });
        inner_pass.set_pipeline(description.get_pipeline());
        Description::set_frame_bind_groups(&mut inner_pass, frame_bind_groups);
        ComputePass {
            inner_pass,
            _desc: std::marker::PhantomData,
        }
    }

    pub fn dispatch(
        &mut self,
        bind_groups: &Description::PassBindGroups,
        size: (u32, u32, u32),
        workgroup_size: (u32, u32, u32),
    ) {
        let inner_pass = &mut self.inner_pass;
        Description::set_pass_bind_groups(inner_pass, bind_groups);
        inner_pass.dispatch(
            size.0 / workgroup_size.0 + size.0 % workgroup_size.0,
            size.1 / workgroup_size.1 + size.1 % workgroup_size.1,
            size.2 / workgroup_size.2 + size.2 % workgroup_size.2,
        );
    }
}
