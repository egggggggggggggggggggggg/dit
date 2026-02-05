use math::{lalg::Vec2, shape::Shape};

use crate::edge_select::DistanceSelector;

trait ContourCombiner {
    type Selector: DistanceSelector;
    fn new(shape: Shape) -> Self;
    //Distance type is constrained to the type of the selector via an assertion
    fn distance(&mut self) -> <Self::Selector as DistanceSelector>::DistanceType;
    fn reset(&mut self, p: Vec2);
    fn edge_selector(&mut self) -> &mut Self::Selector;
}

pub struct SimpleContourCombiner<T: DistanceSelector> {
    shape_edge_selector: T,
}
impl<T: DistanceSelector> ContourCombiner for SimpleContourCombiner<T> {
    type Selector = T;
    fn distance(&mut self) -> <Self::Selector as DistanceSelector>::DistanceType {
        let edge_selector = self.edge_selector();
        let distance: <T as DistanceSelector>::DistanceType = edge_selector.distance();
        distance
    }
    fn edge_selector(&mut self) -> &mut Self::Selector {
        &mut self.shape_edge_selector
    }
    fn new(shape: Shape) -> Self {
        Self {
            shape_edge_selector: Self::Selector::new(),
        }
    }
    fn reset(&mut self, p: Vec2) {}
}
