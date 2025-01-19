#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Uint24_8(u32);

impl Uint24_8 {
    pub fn new(value_24: u32, value_8: u8) -> Self {
        Self {
            0: (value_8 as u32) << 24 | value_24,
        }
    }

    pub fn value_24(&self) -> u32 {
        self.0 & 0x00FFFFFF
    }

    pub fn value_8(&self) -> u8 {
        ((self.0 & 0xFF000000) >> 24) as u8
    }
}
