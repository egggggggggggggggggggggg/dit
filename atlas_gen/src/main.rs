use atlas_gen::{
    allocator::ShelfAllocator,
    atlas::Atlas,
    cont_comb::{ContourCombiner, SimpleContourCombiner},
    distances::MultiDistance,
    edge_coloring::edge_coloring_simple,
    edge_select::MultiDistanceSelector,
    shape_distance_finder::ShapeDistanceFinder,
};
use font_parser::TtfFont;
use image::{ImageBuffer, Rgb};
use math::{bezier::Bounds, calc::Range, lalg::Vec2, shape::Shape};

fn main() {
    entry();
}
const CROSS_THRESHOLD: f64 = 3.0;
fn entry() {
    let mut font = TtfFont::new("../JetBrainsMonoNerdFontMono-Regular.ttf").unwrap();
    let atlas_allocator = ShelfAllocator::new(512, 512);
    let mut texture_atlas: Atlas<char, Rgb<u8>, ShelfAllocator> =
        Atlas::new(1024, 1024, atlas_allocator, 4, false);
    let target_font_px = 64;
    let mut seed = 10000000;
    for ch in '!'..'~' {
        let gid = font.lookup(ch as u32).unwrap();
        let mut shape = font.assemble_glyf(gid as u16).unwrap();

        edge_coloring_simple(&mut shape, CROSS_THRESHOLD.sin(), &mut seed);
        println!("shape {:?}", shape);
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
        let mut x_direction = 1;
        for y in 0..output_image.height() {
            let mut x = if x_direction < 0 {
                output_image.width() as i32 - 1
            } else {
                0
            };
            for col in 0..output_image.width() {
                let p = Vec2 {
                    y: y as f64 / output_image.height() as f64,
                    x: col as f64 / output_image.width() as f64,
                };
                println!("point: {:?}", p);
                let distance = sdf.distance(p);
                println!("distance: {:?}", distance);
                x += x_direction;
            }
            x_direction = -x_direction
        }
        break;
    }
}
//gotta add some info to the thing for it to properly use it
pub struct GlyphGeometry {
    index: i64,
    codepoint: u16,
    geometry_scale: f64,
    shape: Shape,
    bounds: Bounds,
    advance: f64,
}
pub struct Box {
    rect: Rectangle,
    range: Range,
    scale: f64,
    translate: Vec2,
    outter_padding: Padding,
}
pub struct Padding {
    l: f64,
    b: f64,
    r: f64,
    t: f64,
}
pub struct Rectangle {
    x: i64,
    y: i64,
    w: i64,
    h: i64,
}
pub struct OrientedRectangle {
    rect: Rectangle,
    oriented: bool,
}
