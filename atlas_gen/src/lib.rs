mod allocator;
pub mod atlas;
pub mod math;
pub mod msdfgen;
use std::{f32::EPSILON, vec};

use atlas::*;
use font_parser::{BezierCurve, GlyphHeader, TtfFont, Vec2};
use image::{ImageBuffer, Pixel, Rgb, Rgba};

use crate::{
    allocator::ShelfAllocator,
    math::{Polynomial, Range, bisection},
};
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
            draw_msdf_glyph(contour, 128, font.head.units_per_em, header);
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
    for x in 0..pixel_width {
        for y in 0..pixel_height {
            let mut p = Vec2 {
                x: (x as f32 + 0.5) / pixel_width as f32,
                y: (y as f32 + 0.5) / pixel_height as f32,
            };
            generate_pixel(p, &contour);
        }
    }
    img
}
fn generate_pixel(p: Vec2, shape: &Vec<Vec<BezierCurve>>) {
    let mut min_dist = f32::MAX;
    let mut min_ortho = f32::MAX;
    for contour in shape {
        for curve in contour {
            let (dvec, t) = dist(&p, &curve);
            if dvec.magnitude() < min_dist {
                let sign = (curve.derive_curve().evaluate_bezier(t) * dvec).signum();
                min_dist = dvec.magnitude() * sign;
                min_ortho = orthogonality(&curve, &p);
            } else if dvec.magnitude() == min_dist {
                let ortho = orthogonality(&curve, &p);
                if ortho < min_ortho {
                    let sign = (curve.derive_curve().evaluate_bezier(t) * dvec).signum();
                    min_dist = dvec.magnitude() * sign;
                    min_ortho = ortho;
                }
            }
        }
    }
    println!("dist: {}, ortho: {}", min_dist, min_ortho);
}
const SAMPLE_AMOUNT: u32 = 40;
fn signed_distance(p: Vec2, edge: BezierCurve) {}
fn dist(p: &Vec2, edge: &BezierCurve) -> (Vec2, f32) {
    let distance = match *edge {
        BezierCurve::Quadratic(p0, p1, p2) => {
            let a = p0 - p1 * 2.0 + p2;
            let b = (p1 - p0) * 2.0;
            let c = p0;
            let asquared = a * a;
            let k3 = asquared * 2.0;
            let k2 = asquared * 3.0;
            let l = c - *p;
            let k1 = (b * b) + (a * (l * 2.0));
            let k0 = b * l;
            let cubic = Polynomial {
                coefficients: vec![k3, k2, k1, k0],
            };
            let mut candidate_intervals: Vec<Range> = vec![];
            let mut i = 0;
            let mut roots = vec![0.0, 1.0];
            while i < SAMPLE_AMOUNT + 1 {
                let first = cubic.eval_horner(i as f32 / SAMPLE_AMOUNT as f32);
                let second = cubic.eval_horner((i + 1) as f32 / SAMPLE_AMOUNT as f32);
                if (first.abs() < EPSILON)
                    || (second.abs() < EPSILON)
                    || (first.signum() != second.signum())
                {
                    candidate_intervals.push(Range {
                        lower: i as f32 / SAMPLE_AMOUNT as f32,
                        higher: (i + 1) as f32 / SAMPLE_AMOUNT as f32,
                    })
                }
                i += 1;
            }
            for i in candidate_intervals {
                if let Some(root) = bisection(&cubic, i) {
                    roots.push(root);
                }
            }
            //find the roots

            let mut min_dist = f32::MAX;
            let mut min_vector = Vec2 { x: 0.0, y: 0.0 };
            let mut valid_root = 0.0;
            for t in roots {
                if t < 0.0 || t > 1.0 {
                    continue;
                }
                let b = BezierCurve::Quadratic(p0, p1, p2).evaluate_bezier(t);
                let dvec = b - *p;
                let d = min_vector.magnitude();
                if min_dist > d {
                    min_vector = dvec;
                    min_dist = d;
                    valid_root = t;
                }
            }
            (min_vector, valid_root)
            //evaluate the roots for the min_dist
        }
        _ => (Vec2 { x: 0.0, y: 0.0 }, 0.0),
    };
    distance
}
fn orthogonality(curve: &BezierCurve, p: &Vec2) -> f32 {
    0.0
}
fn classify_corners() {}
fn calculate_bbox() {}
