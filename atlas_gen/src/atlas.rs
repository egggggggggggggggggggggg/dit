use crate::allocator::AtlasAllocator;
use image::{ImageBuffer, Pixel};
use std::{collections::HashMap, fmt::Debug, hash::Hash};

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
    pub uv_table: HashMap<T, ([f32; 2], [f32; 2])>,
    allocator: A,
    width: u32,
    height: u32,
    padding: u32,
}

impl<T, P, A> Atlas<T, P, A>
where
    T: Hash + Eq + Debug + Copy,
    P: Pixel<Subpixel = u8>,
    A: AtlasAllocator,
{
    pub fn new(width: u32, height: u32, allocator: A, padding: u32) -> Self {
        Self {
            image: ImageBuffer::new(width, height),
            table: HashMap::new(),
            uv_table: HashMap::new(),
            allocator,
            width,
            height,
            padding,
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
        let atlas_entry = AtlasEntry {
            x: x + p,
            y: y + p,
            width: w,
            height: h,
        };
        self.table.insert(key, atlas_entry);
        self.uv_table
            .insert(key, atlas_entry.uv(self.width, self.height));
        Ok(())
    }
    pub fn get_uv(&mut self, key: T) -> ([f32; 2], [f32; 2]) {
        if let Some(uv) = self.uv_table.get(&key) {
            return *uv;
        } else {
            let uv = if let Some(entry) = self.table.get(&key) {
                entry.uv(self.width, self.height)
            } else {
                ([0.0, 0.0], [0.0, 0.0])
            };
            self.uv_table.insert(key, uv);
            uv
        }
    }
}
