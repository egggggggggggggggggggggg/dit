use std::{collections::HashMap, hash::Hash};

use font_parser::TtfFont;
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
        let u1 = (self.x + self.width) as f32 / atlas_width as f32;
        let v0 = self.y as f32 / atlas_height as f32; // top
        let v1 = (self.y + self.height) as f32 / atlas_height as f32; // bottom

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
    //simple cache for now
    pub uv_table: HashMap<T, ([f32; 2], [f32; 2])>,
    allocator: A,
    width: u32,
    height: u32,
    padding: u32,
    bleed: bool,
}
impl<T, P, A> Atlas<T, P, A>
where
    T: Hash + Eq,
    P: Pixel<Subpixel = u8>,
    A: AtlasAllocator,
{
    pub fn new(width: u32, height: u32, allocator: A, padding: u32, bleed: bool) -> Self {
        Self {
            image: ImageBuffer::new(width, height),
            table: HashMap::new(),
            uv_table: HashMap::new(),
            allocator,
            width,
            height,
            padding,
            bleed,
        }
    }
    pub fn add_image(&mut self, key: T, src: &ImageBuffer<P, Vec<u8>>) -> Result<(), &'static str> {
        let (w, h) = src.dimensions();
        let p = self.padding;

        let alloc_w = w + 2 * p;
        let alloc_h = h + 2 * p;

        let (x, y) = self
            .allocator
            .allocate(alloc_w, alloc_h)
            .ok_or("Atlas Full")?;
        for sy in 0..h {
            for sx in 0..w {
                let pixel = *src.get_pixel(sx, sy);
                self.image.put_pixel(x + p + sx, y + p + sy, pixel);
            }
        }
        if self.bleed {
            // Horizontal edge bleed
            for sy in 0..h {
                let left = *src.get_pixel(0, sy);
                let right = *src.get_pixel(w - 1, sy);

                for i in 0..p {
                    self.image.put_pixel(x + i, y + p + sy, left);
                    self.image.put_pixel(x + p + w + i, y + p + sy, right);
                }
            }
            // Vertical edge bleed
            for sx in 0..w {
                let top = *src.get_pixel(sx, 0);
                let bottom = *src.get_pixel(sx, h - 1);

                for i in 0..p {
                    self.image.put_pixel(x + p + sx, y + i, top);
                    self.image.put_pixel(x + p + sx, y + p + h + i, bottom);
                }
            }
            // Corner bleed
            let tl = *src.get_pixel(0, 0);
            let tr = *src.get_pixel(w - 1, 0);
            let bl = *src.get_pixel(0, h - 1);
            let br = *src.get_pixel(w - 1, h - 1);
            for dy in 0..p {
                for dx in 0..p {
                    self.image.put_pixel(x + dx, y + dy, tl);
                    self.image.put_pixel(x + p + w + dx, y + dy, tr);
                    self.image.put_pixel(x + dx, y + p + h + dy, bl);
                    self.image.put_pixel(x + p + w + dx, y + p + h + dy, br);
                }
            }
        }

        self.table.insert(
            key,
            AtlasEntry {
                x: x + p,
                y: y + p,
                width: w,
                height: h,
            },
        );
        Ok(())
    }
    pub fn get_uv(&mut self, key: T) -> ([f32; 2], [f32; 2]) {
        if let Some(uv) = self.uv_table.get(&key) {
            return *uv;
        } else {
            let uv = self.table.get(&key).unwrap().uv(self.width, self.height);
            self.uv_table.insert(key, uv);
            uv
        }
    }

    pub fn serialize_metadata(&mut self, path: &'static str) {}
    // Doesn't change the image just removes the table entry that gives access to it
    // Allocator also removes its entry as a result of this
}
