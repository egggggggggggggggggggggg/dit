use std::{collections::HashMap, hash::Hash};

use image::{ImageBuffer, Pixel};

use crate::allocator::AtlasAllocator;

#[derive(Debug, Clone, Copy)]
pub struct AtlasEntry {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl AtlasEntry {
    pub fn uv(&self, atlas_width: u32, atlas_height: u32) -> ([f32; 2], [f32; 2]) {
        let u0 = self.x as f32 / atlas_width as f32;
        let v0 = self.y as f32 / atlas_height as f32;
        let u1 = (self.x + self.width) as f32 / atlas_width as f32;
        let v1 = (self.y + self.height) as f32 / atlas_height as f32;
        ([u0, v0], [u1, v1])
    }
}
//evictable atlas cache
pub struct Atlas<T, P, A>
where
    T: Hash + Eq,
    P: Pixel<Subpixel = u8>,
    A: AtlasAllocator,
{
    pub image: ImageBuffer<P, Vec<u8>>,
    pub table: HashMap<T, AtlasEntry>,
    allocator: A,
}

impl<T, P, A> Atlas<T, P, A>
where
    T: Hash + Eq,
    P: Pixel<Subpixel = u8>,
    A: AtlasAllocator,
{
    pub fn new(width: u32, height: u32, allocator: A) -> Self {
        Self {
            image: ImageBuffer::new(width, height),
            table: HashMap::new(),
            allocator,
        }
    }
    pub fn add_image(&mut self, key: T, src: &ImageBuffer<P, Vec<u8>>) -> Result<(), &'static str> {
        let (w, h) = src.dimensions();
        let (x, y) = self.allocator.allocate(w, h).ok_or("Atlas Full")?;
        for sy in 0..h {
            for sx in 0..w {
                let p = src.get_pixel(sx, sy);
                self.image.put_pixel(x + sx, y + sy, *p);
            }
        }
        self.table.insert(
            key,
            AtlasEntry {
                x,
                y,
                width: w,
                height: h,
            },
        );
        Ok(())
    }
    pub fn serialize_metadata(&mut self, path: &'static str) {}
    // Doesn't change the image just removes the table entry that gives access to it
    // Allocator also removes its entry as a result of this
}
