use crate::{
    bezier::{Bezier, Bounds},
    contour::{self, Contour},
};
//0.5 * (sqrt5 - 1)
const RATIO: f64 = 0.5 * (2.2360679775 - 1.0);
const DDECONVERGE_OVERSHOOT: f64 = 1.11111111111111111;
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

impl Shape {
    pub fn add_contour(&mut self, contour: Contour) {}
    pub fn normalize(&mut self) {
        for contour in &mut self.contours {
            if contour.edges.len() == 1 {
                let edge_segments = contour.edges[0].split_in_thirds();
                contour.edges.clear();
                contour.edges = edge_segments.to_vec();
            } else if !contour.edges.is_empty() {
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
        let orientations: Vec<f64> = Vec::with_capacity(self.contours.len());
        let intersections: Vec<Intersections> = Vec::new();
        for i in 0..self.contours.len() {}
    }
    pub fn get_y_axis_orientation(&self) {}
    pub fn set_y_axis_orientation(&mut self) {}
}
