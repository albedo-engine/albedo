use guillotiere::{size2, AtlasAllocator};
use std::convert::From;

use crate::renderer::resources::TextureInfoGPU;

#[derive(Debug)]
pub enum TextureError {
    InvalidFormat(String),
}

pub struct TextureSlice<'a> {
    data: &'a [u8],
    width: u32,
    height: u32,
}

impl<'a> TextureSlice<'a> {
    pub fn new(data: &'a [u8], width: u32) -> Result<TextureSlice, TextureError> {
        if data.len() % 4 != 0 {
            return Err(TextureError::InvalidFormat(String::from(
                "texture must have RGBA channels",
            )));
        }
        let elt_count = data.len() as u32;
        let height = elt_count / (4 * width);
        Ok(TextureSlice {
            width,
            height,
            data,
        })
    }
}

pub struct TextureAtlas {
    atlas: Vec<AtlasAllocator>,
    size: u32,
    data: Vec<u8>,
    textures: Vec<TextureInfoGPU>,
}

impl TextureAtlas {
    const COMPONENTS: u32 = 4;
    const MAX_SIZE: u32 = 1 << 24;

    fn create_atlas_allocator(size: u32) -> AtlasAllocator {
        AtlasAllocator::new(size2(size as i32, size as i32))
    }

    pub fn new(size: u32) -> TextureAtlas {
        // Atlas is assumed to be squared with `layers` layer count.
        TextureAtlas {
            data: vec![],
            atlas: vec![],
            textures: vec![],
            size: u32::min(size, Self::MAX_SIZE),
        }
    }

    pub fn layer_data(&self, layer: usize) -> &[u8] {
        let bytes_count = self.bytes_per_atlas();
        let start = layer * bytes_count;
        &self.data[start..(start + bytes_count)]
    }

    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    pub fn textures(&self) -> &Vec<TextureInfoGPU> {
        &self.textures
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn layer_count(&self) -> usize {
        self.atlas.len()
    }

    // @todo: return error if size is too big.
    pub fn add(&mut self, texture: &TextureSlice) -> usize {
        let tex_size = size2(texture.width as i32, texture.height as i32);
        for (i, atlas) in self.atlas.iter_mut().enumerate() {
            match atlas.allocate(tex_size) {
                Some(alloc) => return self.add_texture_data(i, alloc, texture),
                _ => (),
            }
        }
        // No atlas found, allocate a new one.
        let bytes_per_atlas = self.bytes_per_atlas();
        self.atlas.push(Self::create_atlas_allocator(self.size));
        self.data.resize(bytes_per_atlas as usize, 0);
        let alloc = self.atlas.last_mut().unwrap().allocate(tex_size).unwrap();
        self.add_texture_data(self.atlas.len() - 1, alloc, texture)
    }

    pub fn bytes_per_atlas(&self) -> usize {
        (self.size * self.size * Self::COMPONENTS) as usize
    }

    pub fn bytes_per_atlas_row(&self) -> usize {
        (self.size * Self::COMPONENTS) as usize
    }

    fn add_texture_data(
        &mut self,
        atlas_index: usize,
        alloc: guillotiere::Allocation,
        texture: &TextureSlice,
    ) -> usize {
        let rectangle = alloc.rectangle;

        let x = rectangle.min.x as u32;
        let y = rectangle.min.y as u32;
        let width = rectangle.width() as u32;
        let height = rectangle.height() as u32;

        // Copy texture data to atlas.
        let bytes_per_atlas = self.bytes_per_atlas();
        let bytes_per_atlas_row = self.bytes_per_atlas_row();
        let bytes_per_atlas_layer = (atlas_index * bytes_per_atlas) as usize;
        let bytes_per_row = (texture.width * Self::COMPONENTS) as usize;
        for i in 0..texture.height {
            let dst_height_byte_offset = (y + i) as usize * bytes_per_atlas_row;
            let dst_start_byte = (
                bytes_per_atlas_layer +
                dst_height_byte_offset +
                (x * Self::COMPONENTS) as usize
            );
            let src_start_byte = i as usize * bytes_per_row;
            let src_slice = &texture.data[src_start_byte..(src_start_byte + bytes_per_row)];
            self.data[dst_start_byte..(dst_start_byte + bytes_per_row)].copy_from_slice(src_slice);
        }

        self.textures
            .push(TextureInfoGPU::new(atlas_index as u8, x, y, width, height));
        self.textures.len() - 1
    }
}
