use core::panic;

use crate::{distances::DistanceType, edge_selectors::DistanceSelector};
use math::{lalg::Vec2, shape::Shape};
trait ContourCombiner {
    type Distance: DistanceSelector;
    fn new(shape: Shape) -> Self;
    fn distance(&mut self) -> Self::Distance;
    fn reset(&mut self, p: Vec2);
    fn edge_selector(&self) -> &T;
}
pub struct SimpleContourCombiner<T: DistanceSelector> {
    pub shape_edge_selector: T,
}
impl<T: DistanceSelector> SimpleContourCombiner<T> {
    pub fn new(shape: Shape) -> Self {
        Self {
            shape_edge_selector: T::default(),
        }
    }
    pub fn distance(&mut self) -> T::Distance {
        self.shape_edge_selector.distance()
    }

    pub fn reset(&mut self, p: Vec2) {
        self.shape_edge_selector.reset(p);
    }
    pub fn edge_selector(&self) -> &T {
        &self.shape_edge_selector
    }
}
pub struct OverlappingContourCombiner<T: DistanceSelector> {
    pub edge_selectors: Vec<T>,
    pub windings: Vec<i64>,
    pub p: Vec2,
}
impl<T: DistanceSelector> OverlappingContourCombiner<T> {
    pub fn new(shape: &Shape) -> Self {
        let mut windings = Vec::with_capacity(shape.contours.len());
        for contour in &shape.contours {
            windings.push(contour.winding());
        }
        Self {
            edge_selectors: Vec::with_capacity(shape.contours.len()),
            windings,
            p: Vec2::default(),
        }
    }
    pub fn reset(&mut self, p: Vec2) {
        self.p = p;
        for edge_selector in &mut self.edge_selectors {
            edge_selector.reset(p);
        }
    }
    pub fn edge_selector(&self, i: usize) -> &T {
        return &self.edge_selectors[i];
    }
    pub fn distance(&mut self) -> T::Distance {
        let contour_count = self.edge_selectors.len();
        let mut shape_edge_selector = T::default();
        let mut inner_edge_selector = T::default();
        let mut outer_edge_selector = T::default();
        shape_edge_selector.reset(self.p);
        inner_edge_selector.reset(self.p);
        outer_edge_selector.reset(self.p);
        for i in 0..contour_count {
            let edge_distance = self.edge_selectors[i].distance();
            shape_edge_selector.merge(&self.edge_selectors[i]);
            if self.windings[i] > 0 && edge_distance.resolve() >= 0.0 {
                inner_edge_selector.merge(&self.edge_selectors[i]);
            }
            if self.windings[i] < 0 && edge_distance.resolve() <= 0.0 {
                outer_edge_selector.merge(&self.edge_selectors[i]);
            }
        }
        let shape_distance = shape_edge_selector.distance();
        let inner_distance = inner_edge_selector.distance();
        let outer_distance = outer_edge_selector.distance();
        let inner_scalar_distance = inner_distance.resolve();
        let outer_scalar_distance = outer_distance.resolve();
        //temporary solution as the type  isnt concretely defined
        let mut distance = shape_edge_selector.distance();
        let mut winding = 0;
        if inner_scalar_distance >= 0.0
            && inner_scalar_distance.abs() <= outer_scalar_distance.abs()
        {
            distance = inner_distance;
            winding = 1;
            for i in 0..contour_count {
                if self.windings[i] > 0 {
                    let contour_distance = self.edge_selectors[i].distance();
                    if contour_distance.resolve().abs() < outer_scalar_distance.abs()
                        && contour_distance.resolve() > distance.resolve()
                    {
                        distance = contour_distance;
                    }
                }
            }
        } else if outer_scalar_distance <= 0.0
            && outer_scalar_distance.abs() < inner_scalar_distance.abs()
        {
            distance = outer_distance;
            winding = -1;
            for i in 0..contour_count {
                if self.windings[i] < 0 {
                    let contour_distance = self.edge_selectors[i].distance();
                    if contour_distance.resolve().abs() < inner_scalar_distance.abs()
                        && contour_distance.resolve() < distance.resolve()
                    {
                        distance = contour_distance;
                    }
                }
            }
        } else {
            return shape_distance;
        }
        for i in 0..contour_count {
            if self.windings[i] != winding {
                let contour_distance = self.edge_selectors[i].distance();
                if contour_distance.resolve() * distance.resolve() >= 0.0
                    && contour_distance.resolve().abs() < distance.resolve().abs()
                {
                    distance = contour_distance;
                }
            }
        }
        if distance.resolve() == shape_distance.resolve() {
            distance = shape_distance;
        }
        return distance;
    }
}
