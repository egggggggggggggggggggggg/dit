use atlas_gen::{
    allocator::ShelfAllocator,
    atlas::Atlas,
    cont_comb::{ContourCombiner, SimpleContourCombiner},
    distances::MultiDistance,
    edge_select::MultiDistanceSelector,
    shape_distance_finder::ShapeDistanceFinder,
};
use font_parser::TtfFont;
use image::{ImageBuffer, Rgb};
use math::lalg::Vec2;

fn main() {
    entry();
}
fn entry() {
    let mut font = TtfFont::new("../JetBrainsMonoNerdFontMono-Regular.ttf").unwrap();
    let atlas_allocator = ShelfAllocator::new(512, 512);
    let mut texture_atlas: Atlas<char, Rgb<u8>, ShelfAllocator> =
        Atlas::new(1024, 1024, atlas_allocator, 4, false);
    let target_font_px = 32;
    for ch in '!'..'~' {
        let gid = font.lookup(ch as u32).unwrap();
        let shape = font.assemble_glyf(gid as u16).unwrap();
        let glyph = font.glyf.get_glyf(gid as u16).unwrap().clone();
        let bounds = glyph.get_header();
        let mut sdf: ShapeDistanceFinder<SimpleContourCombiner<MultiDistanceSelector>> =
            ShapeDistanceFinder::new(shape);
        let scale = target_font_px as f64 / font.head.units_per_em as f64;
        let width = bounds.x_max - bounds.x_min;
        let height = bounds.y_max - bounds.y_min;
        let pixel_width = (width as f64 * scale).ceil().max(1.0) as u32;
        let pixel_height = (height as f64 * scale).ceil().max(1.0) as u32;
        let output_image: ImageBuffer<Rgb<u8>, Vec<u8>> =
            ImageBuffer::new(pixel_width, pixel_height);
        let x_direction = 1;

        for y in 0..output_image.height() {
            let x = if x_direction < 0 {
                output_image.width() - 1
            } else {
                0
            };
            for col in 0..output_image.width() {
                let p = Vec2 {
                    y: y as f64,
                    x: col as f64,
                };
                let distance = sdf.distance(p);
                println!("distance: {:?}", distance);
            }
        }
    }
}
