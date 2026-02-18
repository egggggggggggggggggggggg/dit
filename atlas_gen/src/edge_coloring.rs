use math::{
    bezier::{Bezier, EdgeColor},
    lalg::Vec2,
    shape::Shape,
};

const ANGLE_THRESHOLD: f64 = 3.0;
const EDGE_COLORS: [EdgeColor; 3] = [EdgeColor::CYAN, EdgeColor::MAGENTA, EdgeColor::YELLOW];

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
    EDGE_COLORS[seed_extract3(seed) as usize]
}

fn switch_color(color: &mut EdgeColor, seed: &mut u64) {
    let shift = 1 + seed_extract2(seed);
    let shifted = color.bits() << shift; // keep as integer

    let result = (shifted | (shifted >> 3)) & EdgeColor::WHITE.bits();

    *color = EdgeColor::from_bits_truncate(result);
}

fn switch_color_banned(color: &mut EdgeColor, seed: &mut u64, banned: EdgeColor) {
    let combined = *color & banned;

    if combined == EdgeColor::RED || combined == EdgeColor::GREEN || combined == EdgeColor::BLUE {
        *color = combined ^ EdgeColor::WHITE;
    } else {
        switch_color(color, seed);
    }
}

pub fn edge_coloring_simple(shape: &mut Shape, angle_threshold: f64, seed: &mut u64) {
    // println!("prior to coloring : {:?}", shape);
    let cross_threshold = angle_threshold.sin();
    let mut color = init_color(seed);
    println!("init color: {}", color);
    let mut corners = Vec::new();
    for contour in &mut shape.contours {
        if contour.edges.is_empty() {
            continue;
        }
        corners.clear();
        let mut prev_direction = contour.edges.last().unwrap().direction(1.0);
        for (index, edge) in contour.edges.iter().enumerate() {
            if is_corner(
                prev_direction.normalize(),
                edge.direction(0.0).normalize(),
                cross_threshold,
            ) {
                corners.push(index);
            }
            prev_direction = edge.direction(1.0);
        }
        if corners.is_empty() {
            println!("No corners");
            println!("previous color: {}", color);
            switch_color(&mut color, seed);
            for edge in &mut contour.edges {
                edge.set_color(color);
            }
            println!("after color: {}", color);
        } else if corners.len() == 1 {
            println!("Teardrop case");
            let mut colors = [EdgeColor::WHITE; 3];
            switch_color(&mut color, seed);
            colors[0] = color;
            colors[1] = EdgeColor::WHITE;
            switch_color(&mut color, seed);
            colors[2] = color;
            let corner = corners[0];
            let m = contour.edges.len();
            if m >= 3 {
                for i in 0..m {
                    let idx = (corner + i) % m;
                    let t = 1 + symmetrical_trichotomy(i as i32, m as i32);
                    contour.edges[idx].set_color(colors[t as usize]);
                }
            } else if m >= 1 {
                let mut parts = Vec::new();
                let first_parts = contour.edges[0].split_in_thirds();
                parts[0 + 3 * corner] = first_parts[0];
                parts[1 + 3 * corner] = first_parts[1];
                parts[2 + 3 * corner] = first_parts[2];
                if contour.edges.len() >= 2 {
                    let second_parts = contour.edges[1].split_in_thirds();
                    parts[0 + 3 * corner] = second_parts[0];
                    parts[1 + 3 * corner] = second_parts[1];
                    parts[2 + 3 * corner] = second_parts[2];
                    parts[0].set_color(colors[0]);
                    parts[1].set_color(colors[0]);
                    parts[2].set_color(colors[1]);
                    parts[3].set_color(colors[1]);
                    parts[4].set_color(colors[2]);
                    parts[5].set_color(colors[2]);
                } else {
                    parts[0].set_color(colors[0]);
                    parts[1].set_color(colors[1]);
                    parts[2].set_color(colors[2]);
                }
                contour.edges.clear();
                for part in parts {
                    contour.edges.push(part);
                }
            }
        } else {
            println!("Regular case");
            let corner_count = corners.len();
            let mut spline = 0;
            let start = corners[0];
            let m = contour.edges.len();
            println!("color prior: {:?}", color);
            switch_color(&mut color, seed);
            println!("color after: {:?}", color);
            let initial_color = color;
            for i in 0..m {
                let index = (start + i) % m;
                if spline + 1 < corner_count && corners[spline + 1] == index {
                    spline += 1;
                    let banned = if spline == corner_count - 1 {
                        initial_color
                    } else {
                        color
                    };
                    switch_color_banned(&mut color, seed, banned);
                }
                contour.edges[index].set_color(color);
                println!("Color in loop: {:?}", color);
            }
        }
    }

    // println!("after coloring : {:?}", shape);
}
fn is_corner(a: Vec2, b: Vec2, cross_threshold: f64) -> bool {
    a.dot(b) <= 0.0 || a.cross(b).abs() > cross_threshold
}

fn symmetrical_trichotomy(position: i32, n: i32) -> i32 {
    ((3.0 + 2.875 * position as f64 / (n - 1) as f64 - 1.4375 + 0.5) as i32) - 3
}
