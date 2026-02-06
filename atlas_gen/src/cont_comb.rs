use math::{
    lalg::Vec2,
    shape::{self, Shape},
};

use crate::{distances::DistanceType, edge_select::DistanceSelector};

pub trait ContourCombiner {
    type Selector: DistanceSelector;
    fn new(shape: &Shape) -> Self;
    //Distance type is constrained to the type of the selector via an assertion
    fn distance(&mut self) -> <Self::Selector as DistanceSelector>::DistanceType;
    fn reset(&mut self, p: <Self::Selector as DistanceSelector>::ResetType);
    fn edge_selector(&mut self, i: usize) -> &mut Self::Selector;
}

pub struct SimpleContourCombiner<T: DistanceSelector> {
    shape_edge_selector: T,
}
impl<T: DistanceSelector> ContourCombiner for SimpleContourCombiner<T> {
    type Selector = T;
    fn distance(&mut self) -> <Self::Selector as DistanceSelector>::DistanceType {
        let edge_selector = self.edge_selector(0);
        let distance: <T as DistanceSelector>::DistanceType = edge_selector.distance();
        println!("distance in scc: {:?}", distance);
        distance
    }
    fn edge_selector(&mut self, i: usize) -> &mut Self::Selector {
        &mut self.shape_edge_selector
    }
    fn new(shape: &Shape) -> Self {
        Self {
            shape_edge_selector: Self::Selector::new(),
        }
    }
    fn reset(&mut self, p: T::ResetType) {
        self.shape_edge_selector.reset(p);
    }
}
pub struct OverlappingContourCombiner<T: DistanceSelector> {
    pub edge_selectors: Vec<T>,
    pub windings: Vec<i64>,
    pub p: T::ResetType,
}
impl<T: DistanceSelector> ContourCombiner for OverlappingContourCombiner<T> {
    type Selector = T;
    fn distance(&mut self) -> <Self::Selector as DistanceSelector>::DistanceType {
        let contour_count = self.edge_selectors.len();
        let mut shape_edge_selector = T::new();
        let mut inner_edge_selector = T::new();
        let mut outer_edge_selector = T::new();
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
        println!(
            "s: {:?}, i: {:?}, o: {:?}",
            shape_distance, inner_distance, outer_distance
        );
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
    fn edge_selector(&mut self, i: usize) -> &mut Self::Selector {
        &mut self.edge_selectors[i]
    }
    fn new(shape: &Shape) -> Self {
        let mut windings = Vec::with_capacity(shape.contours.len());
        let mut edge_selectors = Vec::with_capacity(shape.contours.len());
        for contour in &shape.contours {
            windings.push(contour.winding());
            edge_selectors.push(T::default());
        }
        Self {
            edge_selectors,
            windings,
            p: T::ResetType::default(),
        }
    }
    fn reset(&mut self, p: <Self::Selector as DistanceSelector>::ResetType) {
        self.p = p;
        for edge_selector in &mut self.edge_selectors {
            edge_selector.reset(p);
        }
    }
}
