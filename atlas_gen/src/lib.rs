mod allocator;
pub mod atlas;
pub mod math;
use atlas::*;
use font_parser::{BezierCurve, GlyphHeader, TtfFont};
use image::{ImageBuffer, Pixel, Rgb, Rgba};

use crate::allocator::ShelfAllocator;
pub fn entry() {
    let mut font = TtfFont::new("../JetBrainsMonoNerdFontMono-Regular.ttf").unwrap();
    let atlas_allocator = ShelfAllocator::new(1024, 1024);
    let mut texture_atlas: Atlas<char, Rgb<u8>, ShelfAllocator> =
        Atlas::new(1024, 1024, atlas_allocator);
    for c in 'a'..'z' {
        let gid = font.lookup(c as u32).unwrap();
        let contour = font.assemble_glyf(gid as u16).unwrap();
        let glyph = font.glyf.get_glyf(gid as u16).unwrap().clone();
        let header = glyph.get_header();
        let drawn_glyph: ImageBuffer<Rgb<u8>, Vec<u8>> =
            draw_glyph(contour, 128, font.head.units_per_em, header);
        texture_atlas.add_image(c, &drawn_glyph).unwrap();
    }
    texture_atlas.image.save("../texture_atlas.png").unwrap();
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
    let white = *P::from_slice(&[255u8; 3]);
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
fn draw_msdf_glyph<P>(
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
    let white = *P::from_slice(&[255u8; 3]);

    

    img
}
fn classify_corners() {

}
fn calculate_bbox() {
    
}
