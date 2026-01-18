mod allocator;
pub mod atlas;
use std::time::Instant;
use std::{f32::EPSILON, vec};

use atlas::*;
use font_parser::{GlyphHeader, TtfFont};
use image::{ImageBuffer, Pixel, Rgb, Rgba};
use math::calc::{Polynomial, Range, bisection};
use math::lalg::{BezierCurve, BinaryVector, Vec2};

use crate::allocator::ShelfAllocator;
pub fn entry() {
    let mut font = TtfFont::new("../JetBrainsMonoNerdFontMono-Regular.ttf").unwrap();
    let atlas_allocator = ShelfAllocator::new(1024, 1024);
    let mut texture_atlas: Atlas<char, Rgb<u8>, ShelfAllocator> =
        Atlas::new(1024, 1024, atlas_allocator);
    let current = Instant::now();
    for c in 'a'..'z' {
        let gid = font.lookup(c as u32).unwrap();
        let contour = font.assemble_glyf(gid as u16).unwrap();
        let glyph = font.glyf.get_glyf(gid as u16).unwrap().clone();
        let header = glyph.get_header();
        let drawn_glyph: ImageBuffer<Rgb<u8>, Vec<u8>> =
            draw_msdf_glyph(contour, 32, font.head.units_per_em, header);
        texture_atlas.add_image(c, &drawn_glyph).unwrap();
    }
    println!("time_elapsed: {:?}", current.elapsed().as_millis());
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
    let spread = 4.0; // try 4–16

    for x in 0..pixel_width {
        for y in 0..pixel_height {
            let p = Vec2 {
                x: bounds.x_min as f32 + (x as f32 + 0.5) / scale,
                y: bounds.y_max as f32 - (y as f32 + 0.5) / scale,
            };
            let dist = goober(&p, &contour);
            // normalize to 0..1
            let v = 0.5 + dist / (2.0 * spread);
            let v = v.clamp(0.0, 1.0);
            let gray = (v * 255.0) as u8;
            let pixel = *P::from_slice(&[gray, gray, gray]);
            img.put_pixel(x, y, pixel);
        }
    }
    img
}

fn goober(p: &Vec2, shape: &Vec<Vec<BezierCurve>>) -> f32 {
    let mut min_dist: f32 = f32::MAX;
    let colored_edges = color_edges(shape);
    println!("colored_edges: {:?}", colored_edges);
    for contour in shape {
        for curve in contour {
            let mdist = min_signed_dist(p, curve);
            if min_dist.abs() > mdist.abs() {
                min_dist = mdist;
            }
        }
    }
    min_dist
}
#[derive(Debug)]
struct Edge<'a> {
    color: BinaryVector,
    curve: &'a BezierCurve,
}

// shape -> contours -> edges
fn color_edges(shape: &Vec<Vec<BezierCurve>>) -> Vec<Vec<Edge>> {
    let mut result: Vec<Vec<Edge>> = Vec::with_capacity(shape.len());

    for contour in shape {
        let mut colored_contour = Vec::with_capacity(contour.len());
        let mut current = if contour.len() == 1 {
            BinaryVector::WHITE
        } else {
            BinaryVector::MAGENTA
        };
        for curve in contour {
            colored_contour.push(Edge {
                color: current,
                curve: &curve,
            });
            current = if current == BinaryVector::YELLOW {
                BinaryVector::CYAN
            } else {
                BinaryVector::YELLOW
            };
        }
        result.push(colored_contour);
    }
    result
}

fn min_signed_dist(p: &Vec2, edge: &BezierCurve) -> f32 {
    match *edge {
        BezierCurve::Quadratic(p0, p1, p2) => {
            let a = p0 - p1 * 2.0 + p2;
            let b = (p1 - p0) * 2.0;
            let c = p0;
            let k3 = 2.0 * a.dot(a);
            let k2 = 3.0 * a.dot(b);
            let k1 = b.dot(b) + 2.0 * a.dot(c - *p);
            let k0 = b.dot(c - *p);
            let cubic = Polynomial {
                coefficients: vec![k3, k2, k1, k0],
            };
            let roots = cubic.find_roots(40, 0.0001);
            let min_dist_root = min_dist(edge, *p, roots);
            let min_vector = edge.evaluate_bezier(min_dist_root) - *p;
            let dist = min_vector.magnitude();
            let derivative_bezier = edge.derive_curve().evaluate_bezier(min_dist_root);
            let tangent = derivative_bezier.normalize();
            let normal = Vec2 {
                x: -tangent.y,
                y: tangent.x,
            };
            let sign = normal.dot(min_vector).signum();
            sign * dist
        }
        BezierCurve::Linear(p0, p1) => {
            let d = p1 - p0;
            let t = ((*p - p0).dot(d) / d.dot(d)).clamp(0.0, 1.0);
            let roots = vec![0.0, t, 1.0];
            let min_dist_root = min_dist(edge, *p, roots);
            let min_vector = edge.evaluate_bezier(min_dist_root) - *p;
            let dist = min_vector.magnitude();
            let derivative_bezier = edge.derive_curve().evaluate_bezier(min_dist_root);
            let tangent = derivative_bezier.normalize();
            let normal = Vec2 {
                x: -tangent.y,
                y: tangent.x,
            };
            let sign = normal.dot(min_vector).signum();
            sign * dist
        }
        _ => 0.0,
    }
}
fn min_dist(edge: &BezierCurve, p: Vec2, roots: Vec<f32>) -> f32 {
    let mut best_t = 0.0;
    let mut min_dist = f32::MAX;
    for t in roots {
        if !(0.0..=1.0).contains(&t) {
            continue;
        }
        let v = edge.evaluate_bezier(t) - p;
        let d = v.magnitude();

        if d < min_dist {
            min_dist = d;
            best_t = t;
        }
    }
    best_t
}
