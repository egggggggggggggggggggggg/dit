use std::cmp::Ordering;

use crate::{
    arit::{mix, mixf, sign},
    bezier::{Bezier, BezierTypes, Bounds, CubicBezier},
    contour::{self, Contour},
    lalg::Vec2,
};
//0.5 * (sqrt5 - 1)
const RATIO: f64 = 0.5 * (2.2360679775 - 1.0);
const DECONVERGE_OVERSHOOT: f64 = 1.11111111111111111;
#[derive(Clone, Debug)]
pub struct Shape {
    pub bounds: Bounds,
    pub contours: Vec<Contour>,
}

pub struct Intersections {
    // Scanline intersection is defined as y = some constant so only x is needed
    pub x: f64,
    // Winding number
    pub direction: f64,
    // Which contou it intersects with
    pub contour_index: f64,
}
impl Intersections {
    fn compare(&mut self, b: &Self) -> std::cmp::Ordering {
        self.x.partial_cmp(&b.x).unwrap()
    }
}

const MSDFGEN_CORNER_DOT_EPSILON: f64 = 0.000001;
//Methods defined here are for preprocessing of the shape to prevent weird artifacts later on
//Artifacts = mainly just curve weirdness where it considers the
//inside of a double contoured shape like an o to be outside and vice versa
//Also general pixel mistmatch as a result of the improper distance selecting
impl Shape {
    pub fn add_contour(&mut self, contour: Contour) {}
    pub fn normalize(&mut self) {
        for contour in &mut self.contours {
            if contour.edges.len() == 1 {
                // Split the single edge into 3 parts
                let edge_segments = contour.edges[0].split_in_thirds();
                contour.edges.clear();
                contour.edges.extend(edge_segments);
                //Non empty contour
            } else if !contour.edges.is_empty() {
                let len = contour.edges.len();

                for i in 0..len {
                    let prev_index = if i == 0 { len - 1 } else { i - 1 };

                    let (prev_dir, cur_dir);
                    {
                        let prev_edge = &contour.edges[prev_index];
                        let edge = &contour.edges[i];

                        prev_dir = prev_edge.direction(1.0).normalize();
                        cur_dir = edge.direction(0.0).normalize();
                    }

                    if prev_dir.dot(cur_dir) < MSDFGEN_CORNER_DOT_EPSILON - 1.0 {
                        let factor = DECONVERGE_OVERSHOOT
                            * (1.0
                                - (MSDFGEN_CORNER_DOT_EPSILON - 1.0)
                                    * (MSDFGEN_CORNER_DOT_EPSILON - 1.0))
                                .sqrt()
                            / (MSDFGEN_CORNER_DOT_EPSILON - 1.0);

                        let mut axis = factor * (cur_dir - prev_dir).normalize();

                        if convergent_curve_ordering(&contour.edges[prev_index], &contour.edges[i])
                            < 0
                        {
                            axis = -axis;
                        }

                        deconverge_edge(&mut contour.edges[prev_index], 1, axis.orthogonal(true));
                        deconverge_edge(&mut contour.edges[i], 0, axis.orthogonal(false));
                    }
                }
            }
        }
    }
    //This is probably not needed as all it does it check if the memory of the shape itself initialized which is
    //Only really smth that happens in C++
    pub fn validate(&self) {
        for contour in &self.contours {
            if !contour.edges.is_empty() {
                let corner = contour.edges.last().unwrap().point(1.0);
            }
        }
    }

    pub fn bound(&mut self) {
        for contour in &mut self.contours {
            contour.bound(&mut self.bounds);
            //recurive inclusion of lower item bounds
        }
    }
    pub fn bound_miters(&mut self) {
        const LARGE_VALUE: f64 = 1e240;
    }
    pub fn get_bounds(&mut self) -> Bounds {
        self.bounds
    }
    pub fn scanline() {}
    pub fn edge_count(&self) -> usize {
        let mut total = 0;
        for contour in &self.contours {
            total += contour.edges.len();
        }
        total
    }
    // pub fn orient_contours(&mut self) {
    //     let ratio = 0.5 * (5.0f64.sqrt() - 1.0);
    //     let mut orientations: Vec<i64> = Vec::with_capacity(self.contours.len());
    //     let mut intersections: Vec<Intersection> = Vec::new();
    //     for i in 0..self.contours.len() {
    //         if orientations[i] != 0 && !self.contours[i].edges.is_empty() {
    //             let y0 = self.contours[i].edges.first().unwrap().point(0.0).y;
    //             let mut y1 = y0;
    //             //
    //             let y = mixf(y0, y1, ratio);
    //             let x = [0.0f64; 3];
    //             let dy = [0i64; 3];
    //             for j in 0..self.contours.len() {
    //                 for edge in &self.contours[j].edges {
    //                     let n = edge
    //                 }
    //             }
    //         }
    //     }
    // }
}
pub fn deconverge_edge(edge_holder: &mut BezierTypes, param: i64, vector: Vec2) {
    match edge_holder {
        BezierTypes::Quadratic(quadratic) => {
            let cubic = quadratic.to_cubic();
            *edge_holder = BezierTypes::Cubic(cubic);
        }
        BezierTypes::Cubic(cubic) => match param {
            0 => {
                cubic.p[1] = cubic.p[1] + (cubic.p[1] - cubic.p[0]).length() * vector;
            }
            1 => {
                cubic.p[2] = cubic.p[2] + (cubic.p[2] - cubic.p[3]).length() * vector;
            }
            _ => panic!("Invalid param"),
        },
        _ => return,
    }
}

fn simplify_degenerate_curve(control_points: &mut [Vec2], order: usize) -> usize {
    match order {
        3 => {
            if (control_points[1] == control_points[0] || control_points[1] == control_points[3])
                && (control_points[2] == control_points[0]
                    || control_points[2] == control_points[3])
            {
                control_points[1] = control_points[3];
                1
            } else {
                3
            }
        }
        2 => {
            if control_points[1] == control_points[0] || control_points[1] == control_points[2] {
                control_points[1] = control_points[2];
                1
            } else {
                2
            }
        }
        1 => {
            if control_points[0] == control_points[1] {
                0
            } else {
                1
            }
        }
        _ => order,
    }
}
fn convergent_curve_ordering_core(
    points: &[Vec2],
    corner_index: usize,
    before: usize,
    after: usize,
) -> i64 {
    if before == 0 || after == 0 {
        return 0;
    }
    let corner = points[corner_index];
    let mut a1 = points[corner_index - 1] - corner;
    let mut b1 = points[corner_index + 1] - corner;
    let mut a2 = Vec2::default();
    let mut b2 = Vec2::default();
    let mut a3 = Vec2::default();
    let mut b3 = Vec2::default();
    if before >= 2 {
        a2 = points[corner_index - 2] - points[corner_index - 1] - a1;
    }
    if after >= 2 {
        b2 = points[corner_index + 2] - points[corner_index + 1] - b1;
    }
    if before >= 3 {
        a3 = points[corner_index - 3]
            - points[corner_index - 2]
            - (points[corner_index - 2] - points[corner_index - 1])
            - a2;
        a2 = a2 * 3.0;
    }
    if after >= 3 {
        b3 = points[corner_index + 3]
            - points[corner_index + 2]
            - (points[corner_index + 2] - points[corner_index + 1])
            - b2;
        b2 = b2 * 3.0;
    }
    a1 = a1 * before as f64;
    b1 = b1 * after as f64;
    if !a1.is_zero() && !b1.is_zero() {
        let as_len = a1.length();
        let bs_len = b1.length();

        let d = as_len * a1.cross(b2) + bs_len * a2.cross(b1);
        if d != 0.0 {
            return sign(d) as i64;
        }

        let d = as_len * as_len * a1.cross(b3)
            + as_len * bs_len * a2.cross(b2)
            + bs_len * bs_len * a3.cross(b1);
        if d != 0.0 {
            return sign(d) as i64;
        }

        let d = as_len * a2.cross(b3) + bs_len * a3.cross(b2);
        if d != 0.0 {
            return sign(d) as i64;
        }

        return sign(a3.cross(b3)) as i64;
    }

    if !a1.is_zero() || !b1.is_zero() {
        let s = if a1.is_zero() { 1 } else { -1 };

        if !b1.is_zero() || !a1.is_zero() {
            let d = a3.cross(b1);
            if d != 0.0 {
                return s * sign(d) as i64;
            }

            let d = a2.cross(b2);
            if d != 0.0 {
                return s * sign(d) as i64;
            }

            let d = a3.cross(b2);
            if d != 0.0 {
                return s * sign(d) as i64;
            }

            let d = a2.cross(b3);
            if d != 0.0 {
                return s * sign(d) as i64;
            }

            return s * sign(a3.cross(b3)) as i64;
        }
    }
    let d = a2.length().sqrt() * b3.cross(a2) + b2.length().sqrt() * a3.cross(b2);

    if d != 0.0 {
        return sign(d) as i64;
    }
    sign(a3.cross(b3)) as i64
}
pub fn convergent_curve_ordering(a: &BezierTypes, b: &BezierTypes) -> i64 {
    let mut a_points = a.control_points().to_vec();
    let mut b_points = b.control_points().to_vec();

    let mut a_order = a.degree() as usize;
    let mut b_order = b.degree() as usize;

    if !(1..=3).contains(&a_order) || !(1..=3).contains(&b_order) {
        return 0;
    }

    if a_points[a_order] != b_points[0] {
        return 0;
    }
    a_order = simplify_degenerate_curve(&mut a_points, a_order);
    b_order = simplify_degenerate_curve(&mut b_points, b_order);
    let mut combined = Vec::with_capacity(a_order + 1 + b_order);

    for i in 0..a_order {
        combined.push(a_points[i]);
    }

    let corner_index = combined.len();
    combined.push(a_points[a_order]); // shared corner

    for i in 1..=b_order {
        combined.push(b_points[i]);
    }

    convergent_curve_ordering_core(&combined, corner_index, a_order, b_order)
}

#[derive(Debug)]
struct Intersection {
    x: f64,
    direction: i32,
    contour_index: i32,
}

impl PartialEq for Intersection {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x
    }
}

impl Eq for Intersection {}

impl PartialOrd for Intersection {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.x.partial_cmp(&other.x)
    }
}

impl Ord for Intersection {
    fn cmp(&self, other: &Self) -> Ordering {
        // If you are certain x will never be NaN, unwrap is acceptable.
        self.x.partial_cmp(&other.x).unwrap_or(Ordering::Equal)
    }
}
