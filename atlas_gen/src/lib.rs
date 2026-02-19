pub mod allocator;
pub mod atlas;
pub mod cont_comb;
// pub mod contour_combiner;
pub mod distances;
pub mod edge_cache;
pub mod edge_coloring;
pub mod edge_select;
pub mod shape_distance_finder;
use atlas::*;
use font_parser::TtfFont;
use image::{ImageBuffer, Rgb};
use math::lalg::Vec2;

use crate::allocator::ShelfAllocator;
use crate::cont_comb::SimpleContourCombiner;
use crate::edge_coloring::edge_coloring_simple;
use crate::edge_select::MultiDistanceSelector;
use shape_distance_finder::*;
const CROSS_THRESHOLD: f64 = 3.0;
pub fn entry() -> Atlas<char, Rgb<u8>, ShelfAllocator> {
    let mut font = TtfFont::new("../JetBrainsMonoNerdFontMono-Regular.ttf").unwrap();
    let atlas_allocator = ShelfAllocator::new(512, 512);
    let mut texture_atlas: Atlas<char, Rgb<u8>, ShelfAllocator> =
        Atlas::new(1024, 1024, atlas_allocator, 4, false);
    let target_font_px = 64;
    let mut seed = 12;
    for ch in '!'..'~' {
        let gid = font.lookup(ch as u32).unwrap();
        let mut shape = font.assemble_glyf(gid as u16).unwrap();
        edge_coloring_simple(&mut shape, CROSS_THRESHOLD.sin(), &mut seed);
        let glyph = font.glyf.get_glyf(gid as u16).unwrap().clone();
        let bounds = glyph.get_header();
        let mut sdf: ShapeDistanceFinder<SimpleContourCombiner<MultiDistanceSelector>> =
            ShapeDistanceFinder::new(shape);
        let scale = target_font_px as f64 / font.head.units_per_em as f64;
        let width = bounds.x_max - bounds.x_min;
        let height = bounds.y_max - bounds.y_min;
        let pixel_width = (width as f64 * scale).ceil().max(1.0) as u32;
        let pixel_height = (height as f64 * scale).ceil().max(1.0) as u32;
        let mut output_image: ImageBuffer<Rgb<u8>, Vec<u8>> =
            ImageBuffer::new(pixel_width, pixel_height);
        for py in 0..pixel_height {
            for px in 0..pixel_width {
                let gx = bounds.x_min as f64 + px as f64 / scale;
                let gy = bounds.y_min as f64 + py as f64 / scale;

                let p = Vec2 { x: gx, y: gy };
                let distance = sdf.distance(p);

                let clamped_r = distance.r.clamp(-127.0, 128.0);
                let clamped_g = distance.g.clamp(-127.0, 128.0);
                let clamped_b = distance.b.clamp(-127.0, 128.0);
                let r_0_255 = (clamped_r + 127.0).round() as u8;
                let g_0_255 = (clamped_g + 127.0).round() as u8;
                let b_0_255 = (clamped_b + 127.0).round() as u8;
                let pixel = Rgb([r_0_255, g_0_255, b_0_255]);
                output_image.put_pixel(px, py, pixel);
            }
        }
        texture_atlas.add_image(ch, &output_image).unwrap();
    }
    texture_atlas.image.save("../texture_atlassss.png").unwrap();
    texture_atlas
}
