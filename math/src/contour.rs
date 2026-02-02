use crate::{
    arit::{shoelace, sign},
    bezier::{Bezier, BezierTypes, Bounds},
};

#[derive(Clone)]
pub struct Contour {
    pub edges: Vec<BezierTypes>,
}
impl Contour {
    pub fn add_edge(&mut self, edge: BezierTypes) {
        self.edges.push(edge);
    }
    pub fn bound(&mut self, bounds: &mut Bounds) {
        for edge in &self.edges {
            edge.bound(bounds);
        }
    }
    pub fn bounds_mitered(
        &mut self,
        bounds: &mut Bounds,
        border: f64,
        miter_limit: f64,
        polarity: f64,
    ) {
        if self.edges.is_empty() {
            return;
        }
        let mut prev_dir = self
            .edges
            .last()
            .unwrap()
            .direction(1.0)
            .normalize_allow_zero(true);
        for edge in &self.edges {
            let dir = -(*edge).direction(0.0).normalize_allow_zero(true);
            if polarity * prev_dir.cross(dir) >= 0.0 {
                let mut miter_length = miter_limit;
                let q = 0.5 * (1.0 - prev_dir.dot(dir));
                if q > 0.0 {
                    miter_length = (1.0 / q.sqrt()).min(miter_limit);
                }
                let miter = edge.point(0.0)
                    + ((prev_dir + dir).normalize_allow_zero(true) * (border * miter_length));
                bounds.include_point(miter);
            }
            prev_dir = edge.direction(1.0).normalize_allow_zero(true);
        }
    }
    pub fn winding(&self) -> f64 {
        if self.edges.is_empty() {
            return 0.0;
        }
        let mut total = 0.0;
        if self.edges.len() == 1 {
            let a = self.edges[0].point(0.0);
            let b = self.edges[0].point(1.0 / 3.0);
            let c = self.edges[0].point(2.0 / 3.0);
            total += shoelace(a, b);
            total += shoelace(b, c);
            total += shoelace(c, a);
        } else if self.edges.len() == 2 {
            let a = self.edges[0].point(0.0);
            let b = self.edges[0].point(0.5);
            let c = self.edges[1].point(0.0);
            let d = self.edges[1].point(0.5);
            total += shoelace(a, b);
            total += shoelace(b, c);
            total += shoelace(c, d);
            total += shoelace(d, a);
        } else {
            let mut prev = self.edges.last().unwrap().point(0.0);
            for edge in &self.edges {
                let cur = edge.point(0.0);
                total += shoelace(prev, cur);
                prev = cur;
            }
        }
        sign(total) as f64
    }
    pub fn reverse(&mut self) {
        self.edges.reverse();
        for edge in &mut self.edges {
            edge.reverse();
        }
    }
}
