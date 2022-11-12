pub mod macros;
pub mod passes;
pub mod texture;
pub mod uniforms;

pub use uniforms::*;

pub fn get_dispatch_size(
    size: (u32, u32, u32),
    workgroup_size: (u32, u32, u32),
) -> (u32, u32, u32) {
    (
        size.0 / workgroup_size.0 + size.0 % workgroup_size.0,
        size.1 / workgroup_size.1 + size.1 % workgroup_size.1,
        size.2 / workgroup_size.2 + size.2 % workgroup_size.2,
    )
}
