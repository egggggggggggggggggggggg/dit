use std::{collections::HashMap, hash::Hash};

use font_parser::{BezierCurve, Glyph, GlyphHeader, TtfFont, Vec2};
use image::{ImageBuffer, Pixel, Rgb, Rgba};

use crate::allocator::{AtlasAllocator, ShelfAllocator};
pub mod allocator;
fn main() {
    let mut font = TtfFont::new("../JetBrainsMonoNerdFontMono-Regular.ttf").unwrap();
    let atlas_allocator = ShelfAllocator::new(1024, 1024);
    let mut texture_atlas: Atlas<char, Rgba<u8>, ShelfAllocator> =
        Atlas::new(1024, 1024, atlas_allocator);
    for c in 'a'..'z' {
        let gid = font.lookup(c as u32).unwrap();
        let contour = font.assemble_glyf(gid as u16).unwrap();
        let glyph = font.glyf.get_glyf(gid as u16).unwrap().clone();
        let header = glyph.get_header();
        let drawn_glyph: ImageBuffer<Rgba<u8>, Vec<u8>> =
            draw_glyph(contour, 128, font.head.units_per_em, header);
        texture_atlas.add_image(c, &drawn_glyph).unwrap();
    }
    texture_atlas.image.save("texture_atlas.png").unwrap();
}
fn draw_glyph<P>(
    contour: Vec<Vec<BezierCurve>>,
    font_size: u16,
    units_per_em: u16,
    bounds: &GlyphHeader,
) -> ImageBuffer<P, Vec<u8>>
where
    P: Pixel<Subpixel = u8> + Copy,
{
    let scale = font_size as f32 / units_per_em as f32;
    let width = bounds.x_max - bounds.x_min;
    let height = bounds.y_max - bounds.y_min;
    let pixel_width = (width as f32 * scale).ceil().max(1.0) as u32;
    let pixel_height = (height as f32 * scale).ceil().max(1.0) as u32;
    let mut img = ImageBuffer::new(pixel_width, pixel_height);
    let white = *P::from_slice(&[255u8; 4]);
    for curves in contour {
        for curve in curves {
            for i in 0..100 {
                let t = i as f32 / 100.0;
                let p = curve.evaluate_bezier(t);

                let x = (p.x - bounds.x_min as f32) * scale;
                let y = (bounds.y_max as f32 - p.y) * scale;

                if x >= 0.0 && y >= 0.0 && x < pixel_width as f32 && y < pixel_height as f32 {
                    img.put_pixel(x as u32, y as u32, white);
                }
            }
        }
    }

    img
}

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
struct Atlas<T, P, A>
where
    T: Hash + Eq,
    P: Pixel<Subpixel = u8>,
    A: AtlasAllocator,
{
    image: ImageBuffer<P, Vec<u8>>,
    table: HashMap<T, AtlasEntry>,
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
