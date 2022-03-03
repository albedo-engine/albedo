pub trait ComputePassDescription {
    type FrameBindGroups: Sized;
    type PassBindGroups: Sized;

    fn get_name() -> &'static str;
    fn set_frame_bind_groups(pass: &mut wgpu::ComputePass, groups: &Self::FrameBindGroups);
    fn set_pass_bind_groups(pass: &mut wgpu::ComputePass, groups: &Self::PassBindGroups);

    fn get_workgroup_size(&self) -> (u32, u32, u32);
    fn get_pipeline(&self) -> &wgpu::ComputePipeline;
}

pub struct ComputePass<'a, Description: ComputePassDescription> {
    inner_pass: wgpu::ComputePass<'a>,
    desc: &'a Description,
}

impl<'a, Description> ComputePass<'a, Description> where
    Description: ComputePassDescription,
{
    pub fn new(
        encoder: &'a mut wgpu::CommandEncoder,
        description: &'a Description,
        frame_bind_groups: &Description::FrameBindGroups
    ) -> ComputePass<'a, Description> {
        let mut inner_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some(Description::get_name()),
        });
        inner_pass.set_pipeline(description.get_pipeline());
        Description::set_frame_bind_groups(&mut inner_pass, frame_bind_groups);
        ComputePass {
            inner_pass,
            desc: description,
        }
    }

    pub fn dispatch(
        &mut self,
        bind_groups: &Description::PassBindGroups,
        size: (u32, u32, u32),
    ) {
        let inner_pass = &mut self.inner_pass;
        Description::set_pass_bind_groups(inner_pass, bind_groups);
        let group_size = self.desc.get_workgroup_size();
        inner_pass.dispatch(
            size.0 / group_size.0,
            size.1 / group_size.1,
            size.2 / group_size.2
        );
    }

}
