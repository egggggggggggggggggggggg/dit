use math::{lalg::Vec2, shape::Shape};

use crate::edge_select::DistanceSelector;

trait ContourCombiner {
    type Selector: DistanceSelector;
    fn new(shape: Shape) -> Self;
    //Distance type is constrained to the type of the selector via an assertion
    fn distance(&mut self) -> &<Self::Selector as DistanceSelector>::DistanceType;
    fn reset(&mut self, p: Vec2);
    fn edge_selector(&self) -> &Self::Selector;
}

pub struct SimpleContourCombiner<T: DistanceSelector> {
    shape_edge_selector: T,
}
impl<T: DistanceSelector> ContourCombiner for SimpleContourCombiner<T> {
    type Selector = T;
    fn distance(&mut self) -> &<Self::Selector as DistanceSelector>::DistanceType {
        &self.edge_selector().distance()
    }
    fn edge_selector(&self) -> &Self::Selector {
        &self.shape_edge_selector
    }
    fn new(shape: Shape) -> Self {
        Self {
            
        }
    }
    fn reset(&mut self, p: Vec2) {}
}
