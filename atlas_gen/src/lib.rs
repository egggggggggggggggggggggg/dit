pub mod allocator;
pub mod atlas;
use core::{f32, panic};
use std::time::Instant;
use std::{f32::EPSILON, vec};

use atlas::*;
use font_parser::{GlyphHeader, TtfFont};
use image::{ImageBuffer, Pixel, Rgb, Rgba};
use math::calc::{Polynomial, Range, bisection, median};
use math::lalg::{BezierCurve, BinaryVector, Vec2};

use crate::allocator::ShelfAllocator;
pub fn entry() -> Atlas<char, Rgb<u8>, ShelfAllocator> {
    let mut font = TtfFont::new("../JetBrainsMonoNerdFontMono-Regular.ttf").unwrap();
    let atlas_allocator = ShelfAllocator::new(512, 512);
    let mut texture_atlas: Atlas<char, Rgb<u8>, ShelfAllocator> =
        Atlas::new(512, 512, atlas_allocator);
    let current = Instant::now();
    for c in '!'..'~' {
        let gid = font.lookup(c as u32).unwrap();
        let contour = font.assemble_glyf(gid as u16).unwrap();
        let glyph = font.glyf.get_glyf(gid as u16).unwrap().clone();
        let header = glyph.get_header();
        let drawn_glyph: ImageBuffer<Rgb<u8>, Vec<u8>> =
            draw_msdf_glyph(contour, 32, font.head.units_per_em, header);
        texture_atlas.add_image(c, &drawn_glyph).unwrap();
    }
    texture_atlas.image.save("../texture_atlas2.png").unwrap();
    texture_atlas
}
fn sample_atlas(texture_atlas: &Atlas<char, Rgb<u8>, ShelfAllocator>, char: char) {
    if let Some(entry) = texture_atlas.table.get(&char) {
        let mut char_image: ImageBuffer<Rgb<u8>, Vec<u8>> =
            ImageBuffer::new(entry.width, entry.height);
        for x in 0..entry.width {
            for y in 0..entry.height {
                let px = entry.x + x;
                let py = entry.y + y;
                let new_pixel = *texture_atlas.image.get_pixel(px, py);
                let Rgb([r, g, b]) = new_pixel;

                let sd = median(r, g, b);
                println!("sd: {}", sd);
                char_image.put_pixel(x, y, Rgb([sd, sd, sd]));
            }
        }
        char_image.save(format!("../letters/{}.png", char)).unwrap();
        println!("{:?}", entry);
    };
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
    let colored_shape = color_edges(&contour);
    for x in 0..pixel_width {
        for y in 0..pixel_height {
            let p = Vec2 {
                x: bounds.x_min as f32 + (x as f32 + 0.5) / scale,
                y: bounds.y_max as f32 - (y as f32 + 0.5) / scale,
            };
            let dr = closest_edge_by_color(&p, &colored_shape, BinaryVector::RED);
            let dg = closest_edge_by_color(&p, &colored_shape, BinaryVector::GREEN);
            let db = closest_edge_by_color(&p, &colored_shape, BinaryVector::BLUE);

            let r = ((0.5 + dr / (2.0 * spread)).clamp(0.0, 1.0) * 255.0) as u8;
            let g = ((0.5 + dg / (2.0 * spread)).clamp(0.0, 1.0) * 255.0) as u8;
            let b = ((0.5 + db / (2.0 * spread)).clamp(0.0, 1.0) * 255.0) as u8;
            img.put_pixel(x, y, *P::from_slice(&[r, g, b]));
        }
    }
    img
}
#[derive(Debug)]
struct Edge<'a> {
    pub color: BinaryVector,
    pub curve: &'a BezierCurve,
}
// shape -> contours -> edges
fn color_edges<'a>(shape: &'a Vec<Vec<BezierCurve>>) -> Vec<Vec<Edge<'a>>> {
    let mut result: Vec<Vec<Edge<'a>>> = Vec::with_capacity(shape.len());
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
const COLORS: [BinaryVector; 3] = [
    BinaryVector::YELLOW,  // R+G
    BinaryVector::CYAN,    // G+B
    BinaryVector::MAGENTA, // R+B
];
fn color_edges2<'a>(shape: &'a Vec<Vec<BezierCurve>>) -> Vec<Vec<Edge<'a>>> {
    let mut result = Vec::with_capacity(shape.len());

    for contour in shape {
        let mut colored = Vec::with_capacity(contour.len());

        for (i, curve) in contour.iter().enumerate() {
            colored.push(Edge {
                color: COLORS[i % 3],
                curve,
            });
        }

        result.push(colored);
    }

    result
}
///Finds the best candidate edge for
fn closest_edge_by_color(p: &Vec2, shape: &Vec<Vec<Edge>>, color: BinaryVector) -> f32 {
    let mut best: Option<Candidate> = None;
    for contour in shape {
        for edge in contour {
            if edge.color.dot(&color) == 0 {
                continue;
            }
            let t = min_dist(edge.curve, *p);
            let candidate = categorize(p, edge.curve, t);
            best = match best {
                None => Some(candidate),
                Some(prev) => {
                    if cmp(&candidate, &prev) {
                        Some(candidate)
                    } else {
                        Some(prev)
                    }
                }
            };
        }
    }

    let mut c = best.unwrap();
    c.dist *= signed_dist(c.t, &c.edge, p);
    c.dist
}
fn generate_pixel(p: &Vec2, shape: &Vec<Vec<Edge>>) {
    let mut d_red = f32::INFINITY;
    let mut d_green = f32::INFINITY;
    let mut d_blue = f32::INFINITY;
    for contour in shape {
        for edge in contour {
            let t = min_dist(edge.curve, *p);

            if edge.color.dot(&BinaryVector::RED) != 0 {}
            if edge.color.dot(&BinaryVector::GREEN) != 0 {}
            if edge.color.dot(&BinaryVector::BLUE) != 0 {}
        }
    }
}
#[derive(Copy, Clone, Debug)]
struct Candidate {
    dist: f32,
    ortho: f32,
    t: f32,
    edge: BezierCurve,
}
const DIST_EPS: f32 = 1e-4;
const ORTHO_EPS: f32 = 1e-3;
///Finds the best candidate aka the one with either smaller absolute distance
///or if they're equal the more orthogonal wins
/// If still tied, go for the one thats more interior aka within the range 0 < t < 1
fn cmp(a: &Candidate, b: &Candidate) -> bool {
    if (a.dist - b.dist).abs() > DIST_EPS {
        return a.dist < b.dist;
    }
    if (a.ortho - b.ortho).abs() > ORTHO_EPS {
        return a.ortho < b.ortho;
    }
    a.t > 0.0 && a.t < 1.0
}

///Gives the sign for the distance value once it has been found
fn signed_dist(min_dist_root: f32, edge: &BezierCurve, p: &Vec2) -> f32 {
    let min_vector = edge.evaluate_bezier(min_dist_root) - *p;
    let derirative_bezier = edge.derive_curve().evaluate_bezier(min_dist_root);
    let tangent = derirative_bezier.normalize();
    let normal = Vec2 {
        x: -tangent.y,
        y: tangent.x,
    };
    normal.dot(min_vector).signum()
}
///Gives information for the cmp function to work with (orthogonality values)
fn categorize(p: &Vec2, edge: &BezierCurve, min_dist_root: f32) -> Candidate {
    let min_vector = edge.evaluate_bezier(min_dist_root) - *p;
    let dist = min_vector.magnitude();
    let derirative_bezier = edge.derive_curve().evaluate_bezier(min_dist_root);
    let tangent = derirative_bezier.normalize();
    let ortho = tangent.dot(min_vector.normalize()).abs();
    Candidate {
        dist,
        ortho,
        t: min_dist_root,
        edge: *edge,
    }
}
///Gives all the roots for the polynomial resulting from bn(t) - p
fn find_roots(p: &Vec2, edge: &BezierCurve) -> Vec<f32> {
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
            cubic.find_roots(40, 0.0001)
        }
        BezierCurve::Linear(p0, p1) => {
            let d = p1 - p0;
            let t = ((*p - p0).dot(d) / d.dot(d)).clamp(0.0, 1.0);
            vec![0.0, t, 1.0]
        }
        _ => panic!("Cubic was in here for some reason"),
    }
}
///Finds the best root that yields the min dist for an edge and p with the given roots
fn min_dist(edge: &BezierCurve, p: Vec2) -> f32 {
    let roots = find_roots(&p, edge);
    let mut best_root = 0.0;
    let mut min_dist = f32::MAX;
    for t in roots {
        if !(0.0..=1.0).contains(&t) {
            continue;
        }
        let v = edge.evaluate_bezier(t) - p;
        let d = v.magnitude();

        if d < min_dist {
            min_dist = d;
            best_root = t;
        }
    }
    best_root
}

//get the shape
//get the point
//find the min dist by solving the cubic
//get the root or the vector for said min_dist
//maybe try and cache some of the calculations
//get the vector result of the curve when solved at the root
//take the root and solve for the orthogonality
//put it into a candidate struct or smth
//find the best candidate based off of the ortho measure
//take that and do some edge coloring
//determine the pixel color based off of best edge
//blah blah some stuff

// fn min_signed_dist(p: &Vec2, edge: &BezierCurve) -> Candidate {
//     match *edge {
//         BezierCurve::Quadratic(p0, p1, p2) => {
//             let a = p0 - p1 * 2.0 + p2;
//             let b = (p1 - p0) * 2.0;
//             let c = p0;
//             let k3 = 2.0 * a.dot(a);
//             let k2 = 3.0 * a.dot(b);
//             let k1 = b.dot(b) + 2.0 * a.dot(c - *p);
//             let k0 = b.dot(c - *p);
//             let cubic = Polynomial {
//                 coefficients: vec![k3, k2, k1, k0],
//             };
//             let roots = cubic.find_roots(40, 0.0001);
//             let min_dist_root = min_dist(edge, *p, roots);
//             let min_vector = edge.evaluate_bezier(min_dist_root) - *p;
//             let dist = min_vector.magnitude();
//             let derivative_bezier = edge.derive_curve().evaluate_bezier(min_dist_root);
//             let tangent = derivative_bezier.normalize();
//             let normal = Vec2 {
//                 x: -tangent.y,
//                 y: tangent.x,
//             };
//             let sign = normal.dot(min_vector).signum();
//             let ortho = tangent.dot(min_vector.normalize()).abs();
//             Candidate {
//                 dist: dist * sign,
//                 ortho,
//                 t: min_dist_root,
//                 edge: *edge,
//             }
//         }
//         BezierCurve::Linear(p0, p1) => {
//             let d = p1 - p0;
//             let t = ((*p - p0).dot(d) / d.dot(d)).clamp(0.0, 1.0);
//             let roots = vec![0.0, t, 1.0];
//             let min_dist_root = min_dist(edge, *p, roots);
//             let min_vector = edge.evaluate_bezier(min_dist_root) - *p;
//             let dist = min_vector.magnitude();
//             let derivative_bezier = edge.derive_curve().evaluate_bezier(min_dist_root);
//             let tangent = derivative_bezier.normalize();
//             let normal = Vec2 {
//                 x: -tangent.y,
//                 y: tangent.x,
//             };
//             let sign = normal.dot(min_vector).signum();
//             let ortho = tangent.dot(min_vector.normalize()).abs();
//             Candidate {
//                 dist: dist * sign,
//                 ortho,
//                 t: min_dist_root,
//                 edge: *edge,
//             }
//         }
//         _ => panic!("Cubic was in here for some reason"),
//     }
// }
// fn
