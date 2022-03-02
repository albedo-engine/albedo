pub trait ComputePass<FrameBindGroups, PassBindGroups> {

    fn get_name() -> &'static str;

    fn create_pass(encoder: &mut wgpu::CommandEncoder) -> wgpu::ComputePass {
        encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some(Self::get_name()),
        })
    }

    fn get_workgroup_size(&self) -> (u32, u32, u32);

    fn get_pipeline(&self) -> &wgpu::ComputePipeline;

    fn get_inner_pass_mut(&mut self) -> &mut wgpu::ComputePass;

    fn set_pass_bind_groups(&mut self, bind_groups: &PassBindGroups);

    fn start_pass(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        bind_groups: &FrameBindGroups,
    );

    fn dispatch(
        &mut self,
        bind_groups: &PassBindGroups,
        size: (u32, u32, u32),
    ) {
        // let compute_pass = self.get_inner_pass_mut();
        // let group_size = self.get_workgroup_size();
        // compute_pass.set_pipeline(self.get_pipeline());
        // self.set_pass_bind_groups(bind_groups);
        // @todo: how to deal with hardcoded size.
        // compute_pass.dispatch(
        //     size.0 / group_size.0,
        //     size.1 / group_size.1,
        //     size.2 / group_size.2
        // );
    }

}
