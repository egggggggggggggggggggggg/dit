use std::cmp::Ordering;

use crate::{
    arit::{mix, mixf},
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
    pub x: f64,
    pub direction: f64,
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
                // Handle multiple edges: push apart convergent edge segments
                let mut prev_edge = contour.edges.last().unwrap(); // Reference to last edge
                for edge in &mut contour.edges.clone() {
                    let prev_dir = prev_edge.direction(1.0).normalize();
                    let cur_dir = edge.direction(0.0).normalize();
                    let dot = prev_dir.dot(cur_dir);

                    if dot < MSDFGEN_CORNER_DOT_EPSILON - 1.0 {
                        let factor = DECONVERGE_OVERSHOOT
                            * ((1.0
                                - (MSDFGEN_CORNER_DOT_EPSILON - 1.0)
                                    * (MSDFGEN_CORNER_DOT_EPSILON - 1.0))
                                .sqrt())
                            / (MSDFGEN_CORNER_DOT_EPSILON - 1.0);
                        let mut axis = factor * (cur_dir - prev_dir).normalize();

                        // Invert the axis depending on the curve ordering
                        if convergent_curve_ordering(*prev_edge, *edge) < 0 {
                            axis = -axis;
                        }

                        //this is probably gonna be a failure point
                        prev_edge = &deconverge_edge(*prev_edge, 1, axis.orthogonal(true));
                        *edge = deconverge_edge(*edge, 0, axis.orthogonal(false));
                    }
                    prev_edge = edge;
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
    pub fn orient_contours(&mut self) {
        let ratio = 0.5 * (5.0f64.sqrt() - 1.0);
        let mut orientations: Vec<i64> = Vec::with_capacity(self.contours.len());
        let mut intersections: Vec<Intersection> = Vec::new();
        for i in 0..self.contours.len() {
            if orientations[i] != 0 && !self.contours[i].edges.is_empty() {
                let y0 = self.contours[i].edges.first().unwrap().point(0.0).y;
                let mut y1 = y0;
                //
                let y = mixf(y0, y1, ratio);
                let x = [0.0f64; 3];
                let dy = [0i64; 3];
                for j in 0..self.contours.len() {
                    for edge in &self.contours[j].edges {
                        let n = edge
                    }
                }
            }
        }
    }
    pub fn valdiate_orientation(&mut self) {}
}
pub fn deconverge_edge(edge_holder: BezierTypes, param: i64, vector: Vec2) -> BezierTypes {
    match edge_holder {
        BezierTypes::Quadratic(quadratic) => {
            let cubic = quadratic.to_cubic();
            BezierTypes::Cubic(cubic)
        }

        BezierTypes::Cubic(cubic) => {
            let mut p = cubic.p;
            match param {
                0 => {
                    p[1] = p[1] + (p[1] - p[0]).length() * vector;
                }
                1 => {
                    p[2] = p[2] + (p[2] - p[3]).length() * vector;
                }
                _ => panic!("Invalid param. Must be 0 or 1"),
            }
            BezierTypes::Cubic(CubicBezier {
                p,
                color: cubic.color,
            })
        }
        _ => {
            panic!("This case should never appear, malformed translation of font file most likely")
        }
    }
}
pub fn convergent_curve_ordering(a: BezierTypes, b: BezierTypes) -> i64 {
    let mut control_points = [Vec2::default(); 12];
    let corner = &control_points[4..]; // Points to the 5th element (index 4)
    let a_cp_tmp = &control_points[8..]; // Points to the 9th element (index 8)
    let a_order = a.degree();
    let b_order = b.degree();
    if !(a_order >= 1 && a_order <= 3 && b_order >= 1 && b_order <= 3) {
        // Not implemented - only linear, quadratic, and cubic curves supported
        return 0;
    }
    for i in 0..=a_order {}
    0
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
