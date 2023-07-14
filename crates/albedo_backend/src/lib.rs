pub mod data;
pub mod gpu;
pub mod mesh;

pub struct Alignment2D {
    pub unpadded_bytes_per_row: usize,
    pub padded_bytes_per_row: usize,
}

impl Alignment2D {
    pub fn new(width: usize, bytes_per_pixel: usize, align: usize) -> Self {
        let unpadded_bytes_per_row = width * bytes_per_pixel;
        let padded_bytes_per_row_padding = (align - unpadded_bytes_per_row % align) % align;
        let padded_bytes_per_row = unpadded_bytes_per_row + padded_bytes_per_row_padding;
        Self {
            unpadded_bytes_per_row,
            padded_bytes_per_row,
        }
    }

    pub fn texture_buffer_copy(width: usize, bytes_per_pixel: usize) -> Self {
        Self::new(
            width,
            bytes_per_pixel,
            wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as usize,
        )
    }

    pub fn bytes(&self) -> usize {
        self.unpadded_bytes_per_row
    }

    pub fn padded_bytes(&self) -> usize {
        self.padded_bytes_per_row
    }
}
