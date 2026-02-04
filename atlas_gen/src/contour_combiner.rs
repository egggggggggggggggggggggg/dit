use math::{lalg::Vec2, shape::Shape};

use crate::edge_selectors::DistanceSelector;
fn test<T: DistanceSelector>(a: T) {}
struct SimpleContourCombiner<T: DistanceSelector> {
    shape_edge_selector: T,
}
impl<T: DistanceSelector> SimpleContourCombiner<T> {
    fn new(shape: Shape) -> Self {
        Self {
            shape_edge_selector: DistanceSelector::
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
}
impl<T: DistanceSelector> OverlappingContourCombiner<T> {
    fn new(shape: Shape) -> Self {
        Self {}
    }
}
