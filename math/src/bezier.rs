use crate::{
    arit::{mix, non_zero_sign, sign},
    calc::{Polynomial, solve_cubic, solve_quadratic},
    lalg::Vec2,
};
const MSDFGEN_CUBIC_SEARCH_STARTS: usize = 4;
const MSDFGEN_CUBIC_SEARCH_STEPS: usize = 4;
#[derive(Clone, Copy)]
struct SignedDistance {
    distance: f64,
    dot: f64,
}
impl Default for SignedDistance {
    fn default() -> Self {
        Self {
            distance: -f64::MAX,
            dot: 0.0,
        }
    }
}
impl PartialEq for SignedDistance {
    fn eq(&self, other: &Self) -> bool {
        self.distance.abs() == other.distance.abs() && self.dot == other.dot
    }
}
impl PartialOrd for SignedDistance {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let a = self.distance.abs();
        let b = other.distance.abs();

        match a.partial_cmp(&b)? {
            std::cmp::Ordering::Equal => self.dot.partial_cmp(&other.dot),
            ord => Some(ord),
        }
    }
}
bitflags::bitflags! {
    #[derive(Clone, Copy, Eq, PartialEq, Debug)]
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
#[derive(Clone, Copy)]
///AABB - Think it does work for stuff besides bezier curves
///
pub struct Bounds {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}
impl Bounds {
    fn new(x_min: f64, x_max: f64, y_min: f64, y_max: f64) -> Self {
        Self {
            x_min,
            x_max,
            y_min,
            y_max,
        }
    }
    ///Includes a point within the bb
    fn include_point(&mut self, p: Vec2) {
        if p.x < self.x_min {
            self.x_min = p.x
        };
        if p.y < self.y_min {
            self.y_min = p.y
        };
        if p.x > self.x_max {
            self.x_max = p.x
        };
        if p.y > self.y_max {
            self.y_max = p.y
        };
    }
}

#[derive(Clone, Copy, Debug)]
pub enum BezierTypes {
    Linear(LinearBezier),
    Quadratic(QuadraticBezier),
    Cubic(CubicBezier),
}
impl Bezier for BezierTypes {
    fn point(&self, param: f64) -> Vec2 {
        match self {
            BezierTypes::Linear(b) => b.point(param),
            BezierTypes::Quadratic(b) => b.point(param),
            BezierTypes::Cubic(b) => b.point(param),
        }
    }
    fn direction(&self, param: f64) -> Vec2 {
        match self {
            BezierTypes::Linear(b) => b.direction(param),
            BezierTypes::Quadratic(b) => b.direction(param),
            BezierTypes::Cubic(b) => b.direction(param),
        }
    }
    fn direction_change(&self, param: f64) -> Vec2 {
        match self {
            BezierTypes::Linear(b) => b.direction_change(param),
            BezierTypes::Quadratic(b) => b.direction_change(param),
            BezierTypes::Cubic(b) => b.direction_change(param),
        }
    }
    fn signed_distance(&self, origin: Vec2, param: f64) -> SignedDistance {
        match self {
            BezierTypes::Linear(b) => b.signed_distance(origin, param),
            BezierTypes::Quadratic(b) => b.signed_distance(origin, param),
            BezierTypes::Cubic(b) => b.signed_distance(origin, param),
        }
    }
    fn bound(&self, bounds: &mut Bounds) {
        match self {
            BezierTypes::Linear(b) => b.bound(bounds),
            BezierTypes::Quadratic(b) => b.bound(bounds),
            BezierTypes::Cubic(b) => b.bound(bounds),
        }
    }
    fn reverse(&mut self) {
        match self {
            BezierTypes::Linear(b) => b.reverse(),
            BezierTypes::Quadratic(b) => b.reverse(),
            BezierTypes::Cubic(b) => b.reverse(),
        }
    }
    fn move_start(&mut self, to: Vec2) {
        match self {
            BezierTypes::Linear(b) => b.move_start(to),
            BezierTypes::Quadratic(b) => b.move_start(to),
            BezierTypes::Cubic(b) => b.move_start(to),
        }
    }
    fn move_end(&mut self, to: Vec2) {
        match self {
            BezierTypes::Linear(b) => b.move_end(to),
            BezierTypes::Quadratic(b) => b.move_end(to),
            BezierTypes::Cubic(b) => b.move_end(to),
        }
    }
    fn split_in_thirds(&self) {
        match self {
            BezierTypes::Linear(b) => b.split_in_thirds(),
            BezierTypes::Quadratic(b) => b.split_in_thirds(),
            BezierTypes::Cubic(b) => b.split_in_thirds(),
        }
    }
    fn color(&self) -> EdgeColor {
        match self {
            BezierTypes::Linear(b) => b.color(),
            BezierTypes::Quadratic(b) => b.color(),
            BezierTypes::Cubic(b) => b.color(),
        }
    }
    fn set_color(&mut self, color: EdgeColor) {
        match self {
            BezierTypes::Linear(b) => b.set_color(color),
            BezierTypes::Quadratic(b) => b.set_color(color),
            BezierTypes::Cubic(b) => b.set_color(color),
        }
    }
}

pub trait Bezier: Clone + Copy {
    fn point(&self, param: f64) -> Vec2;
    fn direction(&self, param: f64) -> Vec2;
    fn direction_change(&self, param: f64) -> Vec2;
    fn signed_distance(&self, origin: Vec2, param: f64) -> SignedDistance;
    // fn scanline_intersections(&self, x: [f64; 3], dy: [i32; 3], y: f64) -> i32;
    fn distance_to_perpendicular_distance(
        &self,
        distance: &mut SignedDistance,
        origin: Vec2,
        param: f64,
    ) {
        if param < 0.0 {
            let dir = self.direction(0.0).normalize();
            let aq = origin - self.point(0.0);
            let ts = aq.dot(dir);
            if ts < 0.0 {
                let perpendicular_distance = aq.cross(dir);
                if perpendicular_distance.abs() <= distance.distance.abs() {
                    distance.distance = perpendicular_distance;
                    distance.dot = 0.0;
                }
            }
        } else if param > 1.0 {
            let dir = self.direction(1.0).normalize();
            let bq = origin - self.point(1.0);
            let ts = bq.dot(dir);
            if ts > 0.0 {
                let perpendicular_distance = bq.cross(dir);
                if perpendicular_distance.abs() <= distance.distance.abs() {
                    distance.distance = perpendicular_distance;
                    distance.dot = 0.0;
                }
            }
        }
    }
    fn bound(&self, bounds: &mut Bounds);
    fn reverse(&mut self);
    fn move_start(&mut self, to: Vec2);
    fn move_end(&mut self, to: Vec2);
    fn split_in_thirds(&self);
    fn color(&self) -> EdgeColor;
    fn set_color(&mut self, color: EdgeColor);
}
#[derive(Clone, Copy, Debug)]
pub struct LinearBezier {
    pub color: EdgeColor,
    pub p: [Vec2; 2],
}
#[derive(Clone, Copy, Debug)]
pub struct QuadraticBezier {
    pub color: EdgeColor,
    pub p: [Vec2; 3],
}
#[derive(Clone, Copy, Debug)]
pub struct CubicBezier {
    pub color: EdgeColor,
    pub p: [Vec2; 4],
}
impl LinearBezier {
    pub fn new(p0: Vec2, p1: Vec2) -> Self {
        Self {
            p: [p0, p1],
            color: EdgeColor::WHITE,
        }
    }
    pub fn length(&self) -> f64 {
        (self.p[1] - self.p[0]).length()
    }
}
impl QuadraticBezier {
    pub fn new(p0: Vec2, p1: Vec2, p2: Vec2) -> Self {
        Self {
            p: [p0, p1, p2],
            color: EdgeColor::WHITE,
        }
    }
    pub fn length(&self) -> f64 {
        let ab = self.p[1] - self.p[0];
        let br = self.p[2] - self.p[1] - ab;
        let abab = ab.dot(ab);
        let abbr = ab.dot(br);
        let brbr = br.dot(br);
        let ab_len = abab.sqrt();
        let br_len = brbr.sqrt();
        let crs = ab.cross(br);
        let h = (abab + abbr + abbr + brbr).sqrt();
        (br_len * ((abbr + brbr) * h - abbr * ab_len)
            + crs * crs * ((br_len * h + abbr + brbr) / (br_len * ab_len + abbr)).ln())
            / (brbr * br_len)
    }
}
impl CubicBezier {
    pub fn new(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2) -> Self {
        Self {
            p: [p0, p1, p2, p3],
            color: EdgeColor::WHITE,
        }
    }
}
//Credit to https://github.com/Chlumsky/msdfgen
//Was written in C++ and I rewrote to use in Rust

impl Bezier for LinearBezier {
    fn color(&self) -> EdgeColor {
        return self.color;
    }
    fn set_color(&mut self, color: EdgeColor) {
        self.color = color;
    }
    fn direction(&self, t: f64) -> Vec2 {
        self.p[1] - self.p[0]
    }

    fn reverse(&mut self) {
        let tmp = self.p[0];
        self.p[0] = self.p[1];
        self.p[1] = tmp;
    }
    fn signed_distance(&self, origin: Vec2, param: f64) -> SignedDistance {
        let aq = origin - self.p[0];
        let ab = self.p[1] - self.p[0];
        let param = aq.dot(ab) / ab.dot(ab);
        let eq = self.p[(param > 0.5) as usize] - origin;
        let endpoint_dist = eq.length();
        if param > 0.0 && param < 1.0 {
            let ortho_dist = ab.orthonormal(false, false).dot(aq);
            if ortho_dist.abs() < endpoint_dist {
                return SignedDistance {
                    distance: ortho_dist,
                    dot: 0.0,
                };
            }
        }
        SignedDistance {
            distance: non_zero_sign(aq.cross(ab)) * endpoint_dist,
            dot: ab.normalize().dot(eq.normalize()).abs(),
        }
    }
    fn direction_change(&self, param: f64) -> Vec2 {
        Vec2::default()
    }
    // fn scanline_intersections(&self, x: &mut [f64; 3], dy: &mut [i32; 3], y: f64) -> i32 {
    //     if (y >= self.p[0].y && y < self.p[1].y) || (y >= self.p[1].y && y < self.p[0].y) {
    //         let param = (y - self.p[0].y) / (self.p[1].y - self.p[0].y);
    //         x[0] = mix(self.p[0].x, self.p[1].x, param);
    //         dy[0] = sign(self.p[1].y - self.p[0].y) as i32;
    //     }
    //     0
    // }
    fn move_end(&mut self, to: Vec2) {
        self.p[1] = to;
    }
    fn move_start(&mut self, to: Vec2) {
        self.p[0] = to;
    }
    fn split_in_thirds(&self) {}
    fn bound(&self, bounds: &mut Bounds) {
        bounds.include_point(self.p[0]);
        bounds.include_point(self.p[1]);
    }
    fn point(&self, param: f64) -> Vec2 {
        mix(self.p[0], self.p[1], param)
    }
}
impl Bezier for QuadraticBezier {
    fn direction(&self, param: f64) -> Vec2 {
        let tangent = mix(self.p[1] - self.p[0], self.p[2] - self.p[1], param);
        if tangent.is_zero() {
            return self.p[2] - self.p[0];
        }
        tangent
    }
    fn signed_distance(&self, origin: Vec2, param: f64) -> SignedDistance {
        let qa = self.p[0] - origin;
        let ab = self.p[1] - self.p[0];
        let br = self.p[2] - self.p[1] - ab;
        let a = br.dot(br);
        let b = 3.0 * ab.dot(br);
        let c = 2.0 * ab.dot(ab) + qa.dot(br);
        let d = qa.dot(ab);
        let (solutions, is_infinite) = solve_cubic(a, b, c, d);
        let mut endpoint_dir = self.direction(0.0);
        let mut min_distance = non_zero_sign(endpoint_dir.cross(qa)) * qa.length();
        let mut param = -qa.dot(endpoint_dir) / endpoint_dir.dot(endpoint_dir);
        {
            let distance = (self.p[2] - origin).length();
            if distance < min_distance.abs() {
                endpoint_dir = self.direction(1.0);
                min_distance = non_zero_sign(endpoint_dir.cross(self.p[2] - origin)) * distance;
                param = (origin - self.p[1]).dot(endpoint_dir) / endpoint_dir.dot(endpoint_dir);
            }
        }
        for i in 0..solutions.len() {
            //disqualify solutions out of the 0.0 - 1.0 range
            if solutions[i] > 0.0 && solutions[i] < 1.0 {
                let qe = qa + ab * (2.0 * solutions[i]) + br * (solutions[i] * solutions[i]);
                let distance = qe.length();
                if distance <= min_distance.abs() {
                    min_distance = non_zero_sign((ab + br * solutions[i]).dot(qe)) * distance;
                    param = solutions[i];
                }
            }
        }
        if param >= 0.0 && param <= 1.0 {
            return SignedDistance {
                distance: min_distance,
                dot: 0.0,
            };
        }
        if param < 0.5 {
            SignedDistance {
                distance: min_distance,
                dot: self.direction(0.0).normalize().dot(qa.normalize()).abs(),
            }
        } else {
            SignedDistance {
                distance: min_distance,
                dot: self
                    .direction(1.0)
                    .normalize()
                    .dot((self.p[2] - origin).normalize())
                    .abs(),
            }
        }
    }
    fn direction_change(&self, param: f64) -> Vec2 {
        (self.p[2] - self.p[1]) - (self.p[1] - self.p[0])
    }
    fn bound(&self, bounds: &mut Bounds) {
        bounds.include_point(self.p[0]);
        bounds.include_point(self.p[2]);
        let bot = (self.p[1] - self.p[0]) - (self.p[2] - self.p[1]);
        if bot.x != 0.0 {
            let param = (self.p[1].x - self.p[0].x) / bot.x;
            if param > 0.0 && param < 1.0 {
                bounds.include_point(self.point(param));
            }
        }
        if bot.y != 0.0 {
            let param = (self.p[1].y - self.p[0].y) / bot.y;
            if param > 0.0 && param < 1.0 {
                bounds.include_point(self.point(param));
            }
        }
    }
    fn reverse(&mut self) {
        let tmp = self.p[0];
        self.p[0] = self.p[2];
        self.p[2] = tmp;
    }
    fn move_end(&mut self, to: Vec2) {
        let orig_end_dir = self.p[2] - self.p[1];
        let orig_p1 = self.p[1];
        let num = (self.p[2] - self.p[1]).cross(to - self.p[2]);
        let den = (self.p[2] - self.p[1]).cross(self.p[0] - self.p[1]);
        self.p[1] = self.p[1] + ((self.p[0] - self.p[1]) * (num / den));
        self.p[2] = to;
        if orig_end_dir.dot(self.p[2] - self.p[1]) < 0.0 {
            self.p[1] = orig_p1;
        }
    }
    fn move_start(&mut self, to: Vec2) {
        let orig_s_dir = self.p[0] - self.p[1];
        let orig_p1 = self.p[1];

        let num = (self.p[0] - self.p[1]).cross(to - self.p[0]);
        let den = (self.p[0] - self.p[1]).cross(self.p[2] - self.p[1]);
        let factor = num / den;
        self.p[1] = self.p[1] + (self.p[2] - self.p[1]) * factor;
        self.p[0] = to;
        if orig_s_dir.dot(self.p[0] - self.p[1]) < 0.0 {
            self.p[1] = orig_p1;
        }
    }
    fn color(&self) -> EdgeColor {
        self.color
    }
    fn set_color(&mut self, color: EdgeColor) {
        self.color = color;
    }
    fn split_in_thirds(&self) {}
    fn point(&self, param: f64) -> Vec2 {
        mix(
            mix(self.p[0], self.p[1], param),
            mix(self.p[1], self.p[2], param),
            param,
        )
    }
}
impl Bezier for CubicBezier {
    fn direction(&self, param: f64) -> Vec2 {
        let tangent = mix(
            mix(self.p[1], self.p[0], param),
            mix(self.p[2], self.p[1], param),
            param,
        );
        if tangent.is_zero() {
            if param == 0.0 {
                return self.p[2] - self.p[0];
            }
            if param == 1.0 {
                return self.p[3] - self.p[1];
            }
        }
        return tangent;
    }
    fn signed_distance(&self, origin: Vec2, param: f64) -> SignedDistance {
        let qa = self.p[0] - origin;
        let ab = self.p[1] - self.p[0];
        let br = self.p[2] - self.p[1] - ab;
        let ac = (self.p[3] - self.p[2]) - (self.p[2] - self.p[1]) - br;
        let mut endpoint_dir = self.direction(0.0);
        let mut min_distance = non_zero_sign(endpoint_dir.cross(qa)) * qa.length();
        let mut param = -qa.dot(endpoint_dir) / endpoint_dir.dot(endpoint_dir);
        {
            let distance = (self.p[3] - origin).length();
            if distance < min_distance.abs() {
                endpoint_dir = self.direction(1.0);
                min_distance = non_zero_sign(endpoint_dir.cross(self.p[3] - origin)) * distance;
                param = endpoint_dir.dot(endpoint_dir - (self.p[3] - origin))
                    / endpoint_dir.dot(endpoint_dir)
            }
        }
        for i in 0..MSDFGEN_CUBIC_SEARCH_STARTS {
            let mut t = 1.0 / (MSDFGEN_CUBIC_SEARCH_STARTS * i) as f64;
            let t2 = t * t;
            let t3 = t2 * t;
            let mut qe = qa + ab * (3.0 * t) + br * (3.0 * t2) + ac * t3;
            let mut d1 = ab * 3.0 + br * (6.0 * t) + ac * (3.0 * t2);
            let mut d2 = br * 6.0 + ac * (6.0 * t);
            let mut improved_t = t - qe.dot(d1) / (d1.dot(d1) + qe.dot(d2));
            if improved_t > 0.0 && improved_t < 1.0 {
                let mut remaining_steps = MSDFGEN_CUBIC_SEARCH_STEPS;
                loop {
                    t = improved_t;
                    let t2 = t * t;
                    let t3 = t2 * t;
                    qe = qa + ab * (3.0 * t) + br * (3.0 * t2) + ac * t3;
                    d1 = ab * 3.0 + br * (6.0 * t) + ac * (3.0 * t2);
                    remaining_steps -= 1;
                    if remaining_steps == 0 {
                        break;
                    }
                    d2 = br * 6.0 + ac * (6.0 * t);
                    improved_t = t - qe.dot(d1) / (d1.dot(d1) + qe.dot(d2));
                    if !(improved_t > 0.0 && improved_t < 1.0) {
                        break;
                    }
                }
                let distance = qe.length();
                if distance < min_distance.abs() {
                    min_distance = non_zero_sign(d1.cross(qe)) * distance;
                    param = t;
                }
            }
        }
        if param >= 0.0 && param <= 1.0 {
            return SignedDistance {
                distance: min_distance,
                dot: 0.0,
            };
        }
        if param < 0.5 {
            SignedDistance {
                distance: min_distance,
                dot: self.direction(0.0).normalize().dot(qa.normalize()).abs(),
            }
        } else {
            SignedDistance {
                distance: min_distance,
                dot: self
                    .direction(1.0)
                    .normalize()
                    .dot((self.p[3] - origin).normalize())
                    .abs(),
            }
        }
    }
    fn direction_change(&self, param: f64) -> Vec2 {
        mix(
            (self.p[2] - self.p[1]) - (self.p[1] - self.p[0]),
            (self.p[3] - self.p[2]) - (self.p[2] - self.p[1]),
            param,
        )
    }
    fn bound(&self, bounds: &mut Bounds) {
        bounds.include_point(self.p[0]);
        bounds.include_point(self.p[3]);
        let a0 = self.p[1] - self.p[0];
        let a1 = (self.p[2] - self.p[1] - a0) * 2.0;
        let a2 = self.p[3] - (self.p[2] * 3.0) + (self.p[1] * 3.0) - self.p[0];
        let (solutions, _is_infinite) = solve_quadratic(a2.x, a1.x, a0.x);
        for solution in solutions {
            if solution > 0.0 && solution < 1.0 {
                bounds.include_point(self.point(solution));
            }
        }
        let (solutions, _is_infinite) = solve_quadratic(a2.y, a1.y, a0.y);
        for solution in solutions {
            if solution > 0.0 && solution < 1.0 {
                bounds.include_point(self.point(solution));
            }
        }
    }
    fn reverse(&mut self) {
        let mut tmp = self.p[0];
        self.p[0] = self.p[3];
        self.p[3] = tmp;
        tmp = self.p[1];
        self.p[1] = self.p[2];
        self.p[2] = tmp;
    }
    fn move_end(&mut self, to: Vec2) {
        self.p[2] = self.p[2] + to - self.p[3];
        self.p[3] = to;
    }
    fn move_start(&mut self, to: Vec2) {
        self.p[1] = self.p[1] + (to - self.p[0]);
        self.p[0] = to;
    }
    fn color(&self) -> EdgeColor {
        self.color
    }
    fn set_color(&mut self, color: EdgeColor) {
        self.color = color;
    }
    fn split_in_thirds(&self) {}
    fn point(&self, param: f64) -> Vec2 {
        let p12 = mix(self.p[1], self.p[2], param);
        return mix(
            mix(mix(self.p[0], self.p[1], param), p12, param),
            mix(p12, mix(self.p[2], self.p[3], param), param),
            param,
        );
    }
}
