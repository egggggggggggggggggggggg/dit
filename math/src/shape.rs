use crate::{
    bezier::Bounds,
    contour::{self, Contour},
};
//0.5 * (sqrt5 - 1)
const RATIO: f64 = 0.5 * (2.2360679775 - 1.0);
#[derive(Clone)]
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
    pub fn normalize(&mut self) {}
    pub fn validate(&self) {}
    pub fn bound(&mut self) {}
    pub fn bound_miters(&mut self) {}
    pub fn get_bounds(&mut self) -> Bounds {
        self.bounds
    }
    pub fn scanline() {}
    pub fn edge_count(&self) {}
    pub fn orient_contours(&mut self) {
        let orientations: Vec<f64> = Vec::with_capacity(self.contours.len());
        let intersections: Vec<Intersections> = Vec::new();
        for i in 0..self.contours.len() {}
    }
    pub fn get_y_axis_orientation(&self) {}
    pub fn set_y_axis_orientation(&mut self) {}
}
