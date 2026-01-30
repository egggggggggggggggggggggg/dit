use math::lalg::{BezierCurve, BinaryVector, Vec2};
const ANGLE_THRESHOLD: f32 = 3.0;
#[derive(Clone)]
pub struct Edge {
    pub color: EdgeColor,
    pub direction0: Vec2,
    pub direction1: Vec2,
}

#[derive(Clone)]
pub struct Contour {
    pub edges: Vec<Edge>,
}
#[derive(Clone)]
pub struct Shape {
    pub contours: Vec<Contour>,
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Eq, PartialEq)]
    pub struct EdgeColor: u8 {
        const BLACK = 0;
        const RED = 1;
        const GREEN = 2;
        const YELLOW = 3;
        const BLUE = 4;
        const MAGENTA = 5;
        const CYAN = 6;
        const WHITE = 7;
    }
}

fn symmetrical_trichotomy(position: usize, n: usize) -> i32 {
    (3.0 + (2.875 * position as f64) / (n as f64 - 1.0) - 1.4375 + 0.5).round() as i32 - 3
}

impl From<u8> for EdgeColor {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::BLACK,
            1 => Self::RED,
            2 => Self::GREEN,
            3 => Self::YELLOW,
            4 => Self::BLUE,
            5 => Self::MAGENTA,
            6 => Self::CYAN,
            7 => Self::WHITE,
            _ => Self::BLACK,
        }
    }
}

fn seed_extract2(seed: &mut u64) -> i32 {
    let v = (*seed & 1) as i32;
    *seed >>= 1;
    v
}

fn seed_extract3(seed: &mut u64) -> i32 {
    let v = (*seed % 3) as i32;
    *seed /= 3;
    v
}

fn init_color(seed: &mut u64) -> EdgeColor {
    const COLORS: [EdgeColor; 3] = [EdgeColor::CYAN, EdgeColor::MAGENTA, EdgeColor::YELLOW];
    COLORS[seed_extract3(seed) as usize]
}
fn switch_color(color: &mut EdgeColor, seed: &mut u64) -> EdgeColor {
    let shifted = color.bits() << (1 + seed_extract2(seed));
    let white_bits = EdgeColor::WHITE.bits();
    let new_bits = (shifted | shifted >> 3) & white_bits;
    EdgeColor::from_bits(new_bits).unwrap()
}
fn switch_color_with_banned(
    color: &mut EdgeColor,
    seed: &mut u64,
    banned: &mut EdgeColor,
) -> EdgeColor {
    let combined = EdgeColor::from(color.bits() & banned.bits());
    if combined == EdgeColor::RED || combined == EdgeColor::GREEN || combined == EdgeColor::BLUE {
        let combined_bits = combined.bits();
        let white_bits = EdgeColor::WHITE.bits();
        return EdgeColor::from(combined_bits ^ white_bits);
    } else {
        switch_color(color, seed)
    }
}

fn is_corner(a_dir: &Vec2, b_dir: &Vec2, cross_threshold: f32) -> bool {
    a_dir.dot(*b_dir) <= 0.0 || a_dir.cross(*b_dir).abs() > cross_threshold
}

fn edge_coloring_simple(shape: &mut Shape, seed: &mut u64) {
    let cross_threshold = ANGLE_THRESHOLD.cos();
    let mut color = init_color(seed);
    for contour in shape.contours.iter_mut() {
        if contour.edges.is_empty() {
            continue;
        }
        let mut corners = Vec::new();
        let mut prev_direction = contour.edges.last().unwrap().direction1;
        let mut index = 0;
        for edge in &contour.edges {
            let direction = edge.direction0.normalize();
            if is_corner(&prev_direction.normalize(), &direction, cross_threshold) {
                corners.push(index);
            }
            prev_direction = edge.direction1;
            index += 1;
        }
        if corners.is_empty() {
            color = switch_color(&mut color, seed);
            for edge in &mut contour.edges.clone() {
                edge.color = color;
            }
        } else if corners.len() == 1 {
            //single corner
            let mut colors = [EdgeColor::from(0), EdgeColor::from(1), EdgeColor::from(2)];
            color = switch_color(&mut color, seed);
            colors[0] = color;
            colors[1] = EdgeColor::from(7); // White
            color = switch_color(&mut color, seed);
            colors[2] = color;

            let corner = corners[0];
            let m = contour.edges.len();
            if m >= 3 {
                for i in 0..m {
                    let new_color = colors[1 + symmetrical_trichotomy(i, m) as usize];
                    contour.edges[(corner + i) % m].color = new_color;
                }
            } else if m >= 1 {
                // let mut parts = vec![None; 7]; // EdgeSegment parts
                // contour.edges[0].split_in_thirds(&mut parts[0..3]);
                // if m >= 2 {
                //     contour.edges[1].0.split_in_thirds(&mut parts[3..6]);
                //     if let Some(p0) = parts[0] {
                //         p0.color = colors[0];
                //     }
                //     if let Some(p1) = parts[1] {
                //         p1.color = colors[1];
                //     }
                //     if let Some(p2) = parts[2] {
                //         p2.color = colors[2];
                //     }
                // }
                // contour.edges.clear();
                // for part in parts.into_iter().flatten() {
                //     contour.edges.push(EdgeHolder(part));
                // }
            } else {
                let mut corner_count = corners.len();
                let mut spline = 0;
                let mut start = corners[0];
                let mut m = contour.edges.len();
                color = switch_color(&mut color, seed);
                let mut initial_color = color;
                for i in 0..m {
                    let index = (start + i) % m;
                    if spline + 1 < corner_count && corners[spline + 1] == index {
                        spline += 1;
                        let mut banned = EdgeColor::from(
                            (spline == corner_count - 1) as u8 * initial_color.bits(),
                        );
                        color = switch_color_with_banned(&mut color, seed, &mut banned);
                    }
                    contour.edges[index].color = color;
                }
            }
        }
    }
}
