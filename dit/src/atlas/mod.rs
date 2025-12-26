use image::{ImageBuffer, Rgb};

use crate::{
    math::vec::Vec2,
    parse::{TtfFont, table::glyf::*},
};
pub fn entry() {
    let mut ttf = TtfFont::new("../JetBrainsMonoNerdFontMono-Regular.ttf").unwrap();
    let mut test_img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(512, 512);
    assemble_atlas(256, ttf.head.units_per_em, &mut ttf, &mut test_img, 'D');
}
pub fn evaluate_bezier(bcur: &BezierCurve, t: f32) -> Vec2 {
    let u = 1.0 - t;
    match bcur {
        BezierCurve::Cubic(_a, _b, _c, _d) => Vec2 { x: 0.0, y: 0.0 },
        BezierCurve::Quadratic(p0, p1, p2) => *p0 * (u * u) + *p1 * (2.0 * u * t) + *p2 * (t * t),
        BezierCurve::Linear(p0, p1) => (*p0 * u) + (*p1 * t),
    }
}

pub struct GlyphAtlasEntry {
    pub uv_min: Vec2,
    pub uv_max: Vec2,
    pub size_px: Vec2,
    pub bearing_px: Vec2,
    pub advance_px: f32,
}
use std::collections::HashMap;

pub struct FontAtlas {
    pub texture_width: u32,
    pub texture_height: u32,
    pub glyphs: HashMap<u16, GlyphAtlasEntry>,
}
struct RasterGlyph {
    gid: u16,
    bitmap: Vec<u8>,
    width: u32,
    height: u32,
    bearing_x: i32,
    bearing_y: i32,
    advance: f32,
}

pub fn assemble_atlas(
    text_size: u16,
    units_per_em: u16,
    ttf: &mut TtfFont,
    img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    ch: char,
) {
    let scale = text_size as f32 / units_per_em as f32;
    if let Some(gid) = ttf.lookup(ch as u32) {
        let item = ttf.assemble_glyf(gid as u16).unwrap();
        for i in item {
            for curve in i {
                for i in 0..100 {
                    let t = i as f32 / 100.0;
                    let cords = evaluate_bezier(&curve, t) * scale;
                    img.put_pixel(cords.x as u32, cords.y as u32, Rgb([255, 255, 255]));
                }
            }
        }
        img.save("test.png").unwrap();
    } else {
        println!("failed")
    }
}
