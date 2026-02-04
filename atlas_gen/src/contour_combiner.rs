use core::panic;

use math::{calc::median, contour, lalg::Vec2, shape::Shape};

use crate::{distances::DistanceType, edge_selectors::DistanceSelector};
fn test<T: DistanceSelector>(a: T) {}
struct SimpleContourCombiner<T: DistanceSelector> {
    shape_edge_selector: T,
}
impl<T: DistanceSelector>SimpleContourCombiner<T> {
    fn new(shape: Shape) -> Self {
        Self {
            shape_edge_selector: T::default(),
        }
    }
    fn distance(&mut self) {
        self.shape_edge_selector.distance();
    }
    fn reset(&mut self, p: Vec2) {
        self.shape_edge_selector.reset(p);
    }
    fn edge_selector(&self) -> &T {
        &self.shape_edge_selector
    }
}
struct OverlappingContourCombiner<T: DistanceSelector> {
    edge_selectors: Vec<T>,
    windings: Vec<i64>,
    p: Vec2,
}
impl<T: DistanceSelector> OverlappingContourCombiner<T> {
    fn new(shape: Shape) -> Self {
        Self {
            edge_selectors: Vec::new(),
            windings: Vec::new(),
            p: Vec2::default(),
        }
    }
    fn distance<D: DistanceType>(&mut self) -> D {
        let contour_count = self.edge_selectors.len();
        let shape_edge_selector = T::default();
        let inner_edge_selector = T::default();
        let outer_edge_selector = T::default();
        shape_edge_selector.reset(self.p);
        inner_edge_selector.reset(self.p);
        outer_edge_selector.reset(self.p);
        for i in 0..contour_count {
            let edge_distance = self.edge_selectors[i].distance();
            shape_edge_selector.merge(self.edge_selectors[i]);
            if self.windings[i] > 0 && edge_distance.resolve() >= 0.0 {
                inner_edge_selector.merge(self.edge_selectors[i]);
            }
            if self.windings[i] < 0 && edge_distance.resolve() <= 0.0 {
                outer_edge_selector.merge(self.edge_selectors[i]);
            }
        }
        let shape_distance = shape_edge_selector.distance();
        let inner_distance = inner_edge_selector.distance();
        let outer_distance = outer_edge_selector.distance();
        let inner_scalar_distance = inner_distance.resolve();
        let outer_scalar_distance = outer_distance.resolve();
        let mut distance = D::init();
        let mut winding = 0;
        if inner_scalar_distance >= 0.0 && inner_scalar_distance.abs() <= outer_scalar_distance.abs() {
            distance = inner_distance;
            winding = 1;
            for i in 0..contour_count {
                if self.windings[i] > 0 {
                    let contour_distance = self.edge_selectors[i].distance();
                    if contour_distance.resolve().abs() < outer_scalar_distance.abs() && contour_distance.resolve()  > distance.resolve() u{
                        distance = contour_distance;
                    }
                }
            }
        } else if outer_scalar_distance <= 0.0 && outer_scalar_distance.abs() < inner_scalar_distance.abs() {
            distance = outer_distance;
            winding = -1;
            for i in contour_count {
                if self.windings[i] < 0 {
                    
                }
            }
        } 
        else {
        }
        for i in 0..contour_count {
            if self.windings[i] != winding {
                let contour_distance = self.edge_selectors[i].distance();
                if contour_distance.resolve()
            }
        }
        D::default()
    }
}
