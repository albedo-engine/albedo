use guillotiere::{size2, AtlasAllocator, Rectangle};
use std::convert::From;

use crate::renderer::resources;
use albedo_backend::GPUBuffer;
use wgpu;

// @todo: where to put that code so it's cleaner?
impl From<Rectangle> for resources::BoundsGPU {
    fn from(item: Rectangle) -> Self {
        let offset = item.min;
        resources::BoundsGPU(glam::uvec4(
            offset.x as u32,
            offset.y as u32,
            item.width() as u32,
            item.height() as u32,
        ))
    }
}

struct TextureRegion {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

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
        if data.len() % 4 == 0 {
            let elt_count = data.len() as u32;
            let height = elt_count / (4 * width);
            Ok(TextureSlice {
                width,
                height,
                data,
            })
        } else {
            Err(TextureError::InvalidFormat(String::from(
                "texture must have RGBA channels",
            )))
        }
    }
}

pub struct TextureAtlas {
    atlas: Vec<AtlasAllocator>,
    size: u32,
    data: Vec<u8>,
    bounds: Vec<resources::BoundsGPU>,
    texture: Option<wgpu::Texture>,
    bounds_buffer: Option<GPUBuffer<resources::BoundsGPU>>,
}

impl TextureAtlas {
    const COMPONENTS: u32 = 4;

    fn get_absolute_id(atlas_index: u32, id: u32) -> u64 {
        (atlas_index as u64) | (id << (std::mem::size_of::<u64>() / 2)) as u64
    }

    fn extract_id(id: u64) -> (u32, u32) {
        (id as u32, (id >> (std::mem::size_of::<u64>() / 2)) as u32)
    }

    pub fn new(size: u32) -> TextureAtlas {
        // Atlas is assumed to be squared with `layers` layer count.
        TextureAtlas {
            data: vec![],
            atlas: vec![],
            bounds: vec![],
            size,
            texture: None,
            bounds_buffer: None,
        }
    }

    pub fn get_bounds(&self) -> &Vec<resources::BoundsGPU> {
        &self.bounds
    }

    pub fn add(&mut self, texture: &TextureSlice) -> u64 {
        let tex_size = size2(texture.width as i32, texture.height as i32);
        for (i, atlas) in self.atlas.iter_mut().enumerate() {
            match atlas.allocate(tex_size) {
                Some(alloc) => return self.add_texture_data(i as u32, alloc, texture),
                _ => (),
            }
        }
        // No atlas found, allocate a new one.
        let bytes_per_atlas = self.bytes_per_atlas();
        self.atlas.push(AtlasAllocator::new(size2(
            self.size as i32,
            self.size as i32,
        )));
        self.data
            .resize(self.data.len() + bytes_per_atlas as usize, 0);
        let alloc = self.atlas.last_mut().unwrap().allocate(tex_size).unwrap();
        self.add_texture_data((self.atlas.len() - 1) as u32, alloc, texture)
    }

    pub fn bytes_per_atlas(&self) -> usize {
        (self.size * self.size * Self::COMPONENTS) as usize
    }

    pub fn upload(&self, device: &wgpu::Device) {
        // @todo
    }

    fn add_texture_data(&mut self, atlas_index: u32, alloc: guillotiere::Allocation, texture: &TextureSlice) -> u64 {
        let bounds: resources::BoundsGPU = alloc.rectangle.into();
        let id = Self::get_absolute_id(atlas_index, alloc.id.serialize());

        // Copy texture data to atlas.
        let bytes_per_atlas = self.bytes_per_atlas();
        let atlas_width = self.size as usize;
        let tex_height = texture.height as usize;
        let components = Self::COMPONENTS as usize;
        let bytes_per_row = (texture.width * Self::COMPONENTS) as usize;
        let atlas_offset = (atlas_index as usize) * bytes_per_atlas;
        let rectangle_offset = atlas_offset + (bounds.y() * self.size + bounds.x()) as usize;
        for i in 0..(texture.height as usize) {
            let dst_start_byte = (
                rectangle_offset + i * atlas_width
            ) * components;
            let src_start_byte = i * tex_height * components;
            self.data[dst_start_byte..(dst_start_byte + bytes_per_row)]
                .copy_from_slice(&texture.data[src_start_byte..(src_start_byte + bytes_per_row)]);
        }

        self.bounds.push(bounds);

        id
    }

}
