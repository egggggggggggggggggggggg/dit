use math::{
    bezier::{Bezier, BezierTypes, EdgeColor},
    lalg::{BezierCurve, BinaryVector, Vec2},
};
const ANGLE_THRESHOLD: f32 = 3.0;
#[derive(Clone)]
pub struct Contour {
    pub edges: Vec<BezierTypes>,
}
#[derive(Clone)]
pub struct Shape {
    pub contours: Vec<Contour>,
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

fn is_corner(a_dir: &Vec2, b_dir: &Vec2, cross_threshold: f64) -> bool {
    a_dir.dot(*b_dir) <= 0.0 || a_dir.cross(*b_dir).abs() > cross_threshold
}

fn edge_coloring_simple(shape: &mut Shape, seed: &mut u64) {
    let cross_threshold = ANGLE_THRESHOLD.cos() as f64;
    let mut color = init_color(seed);
    for contour in shape.contours.iter_mut() {
        if contour.edges.is_empty() {
            continue;
        }
        let mut corners = Vec::new();
        let mut prev_direction = contour.edges.last().unwrap().direction(1.0);
        let mut index = 0;
        for edge in &contour.edges {
            let direction = edge.direction(0.0).normalize();
            if is_corner(&prev_direction.normalize(), &direction, cross_threshold) {
                corners.push(index);
            }
            prev_direction = edge.direction(1.0);
            index += 1;
        }
        if corners.is_empty() {
            color = switch_color(&mut color, seed);
            for edge in &mut contour.edges.clone() {
                edge.set_color(color);
            }
        } else {
            let corner_count = corners.len();
            let mut spline = 0;
            let start = corners[0];
            let m = contour.edges.len();
            color = switch_color(&mut color, seed);
            let initial_color = color;
            for i in 0..m {
                let index = (start + i) % m;
                if spline + 1 < corner_count && corners[spline + 1] == index {
                    spline += 1;
                    let mut banned =
                        EdgeColor::from((spline == corner_count - 1) as u8 * initial_color.bits());
                    color = switch_color_with_banned(&mut color, seed, &mut banned);
                }
                contour.edges[index].set_color(color);
            }
        }
    }
}
