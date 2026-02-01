use crate::{
    arit::{shoelace, sign},
    bezier::{Bezier, BezierTypes},
};

#[derive(Clone)]
pub struct Contour {
    edges: Vec<BezierTypes>,
}
impl Contour {
    fn add_edge(&mut self, edge: BezierTypes) {
        self.edges.push(edge);
    }
    fn bound(&mut self) {}
    fn bounds_mitered(&mut self) {}
    fn winding(&self) -> f64 {
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
            //Unwrap is safe cuz its nonzero
            let mut prev = self.edges.last().unwrap().point(0.0);
            for edge in &self.edges {
                let cur = edge.point(0.0);
                total += shoelace(prev, cur);
                prev = cur;
            }
        }
        sign(total) as f64
    }
    fn reverse(&mut self) {

    }
}
