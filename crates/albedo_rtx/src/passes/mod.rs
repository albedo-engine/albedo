mod accumulation;
mod blit_pass;
mod debug_bvh_pass;
mod intersector;
mod radiance_estimator;
mod ray;

pub use accumulation::AccumulationPassDescriptor;
pub use blit_pass::BlitPass;
pub use debug_bvh_pass::BVHDebugPass;
pub use intersector::IntersectorPass;
pub use radiance_estimator::ShadingPassDescriptor;
pub use ray::RayPass;

pub struct ShaderSource<'a, Flags> {
    descriptor: wgpu::ShaderModuleDescriptor<'a>,
    flags: Flags,
}
