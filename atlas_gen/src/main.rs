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
use math::{arit::clamp, bezier::Bounds, calc::Range, lalg::Vec2, shape::Shape};

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
    let dmax_px = 1.0;
    for ch in '!'..'~' {
        let mut seed = 0;
        let gid = font.lookup(ch as u32).unwrap();
        let mut shape = font.assemble_glyf(gid as u16).unwrap();
        // edge_coloring_simple(&mut shape, CROSS_THRESHOLD.sin(), &mut seed);

        let glyph = font.glyf.get_glyf(gid as u16).unwrap().clone();
        let bounds = glyph.get_header();

        let mut sdf: ShapeDistanceFinder<SimpleContourCombiner<MultiDistanceSelector>> =
            ShapeDistanceFinder::new(shape);

        // scale = pixels per font unit
        let scale = target_font_px as f64 / font.head.units_per_em as f64;

        // convert dMAX from pixels to font units
        let dmax = dmax_px / scale;
        let distance_range = 2.0 * dmax;
        let max_color = 255.0;

        let width = bounds.x_max - bounds.x_min;
        let height = bounds.y_max - bounds.y_min;

        let pixel_width = (width as f64 * scale).ceil().max(1.0) as u32;
        let pixel_height = (height as f64 * scale).ceil().max(1.0) as u32;

        let mut output_image: ImageBuffer<Rgb<u8>, Vec<u8>> =
            ImageBuffer::new(pixel_width, pixel_height);

        for py in 0..pixel_height {
            for px in 0..pixel_width {
                let gx = bounds.x_min as f64 + (px as f64 + 0.5) / scale;
                let gy = bounds.y_min as f64 + (py as f64 + 0.5) / scale;

                let p = Vec2 { x: gx, y: gy };
                let distance = sdf.distance(p);

                // clamp to [-dmax, +dmax]
                let clamped_r = distance.r.clamp(-dmax, dmax);
                let clamped_g = distance.g.clamp(-dmax, dmax);
                let clamped_b = distance.b.clamp(-dmax, dmax);

                // distanceColor(d) = ((d / (2*dmax)) + 0.5) * 255
                let r_0_255 = ((clamped_r / distance_range + 0.5) * max_color)
                    .clamp(0.0, 255.0)
                    .round() as u8;

                let g_0_255 = ((clamped_g / distance_range + 0.5) * max_color)
                    .clamp(0.0, 255.0)
                    .round() as u8;

                let b_0_255 = ((clamped_b / distance_range + 0.5) * max_color)
                    .clamp(0.0, 255.0)
                    .round() as u8;

                let pixel = Rgb([r_0_255, g_0_255, b_0_255]);

                output_image.put_pixel(px, pixel_height - 1 - py, pixel);
            }
        }

        texture_atlas.add_image(ch, &output_image).unwrap();
    }

    texture_atlas
        .image
        .save("./res/texture_atlas_mono.png")
        .unwrap();
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
