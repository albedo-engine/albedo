mod buffer;
mod compute_pass;

pub mod shader_bindings;

pub use buffer::{GPUBuffer, UniformBuffer};
pub use compute_pass::{ComputePass, ComputePassDescriptor};

pub struct Alignment2D {
    unpadded_bytes_per_row: usize,
    padded_bytes_per_row: usize,
}

impl Alignment2D {
    pub fn new(width: usize, align: usize) -> Self {
        let bytes_per_pixel = std::mem::size_of::<u32>();
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
        let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
        let padded_bytes_per_row = unpadded_bytes_per_row + padded_bytes_per_row_padding;
        Self {
            unpadded_bytes_per_row,
            padded_bytes_per_row,
        }
    }

    pub fn texture_buffer_copy(width: usize) -> Self {
        Self::new(width, wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize)
    }

    pub fn bytes() -> usize {
        self.unpadded_bytes_per_row
    }

    pub fn padded_bytes() -> usize {
        self.padded_bytes_per_row
    }
}
