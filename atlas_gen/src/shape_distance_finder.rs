use math::shape::Shape;

use crate::{cont_comb::ContourCombiner, edge_select::DistanceSelector};

pub struct ShapeDistanceFinder<C: ContourCombiner> {
    pub shape: Shape,
    pub contour_combiner: C,
    pub edge_cache: Vec<<C::Selector as DistanceSelector>::EdgeCache>,
}
impl<C: ContourCombiner> ShapeDistanceFinder<C> {
    pub fn new(shape: Shape) -> Self {
        let cc = C::new(&shape);
        let ec = &shape.edge_count();
        let mut edge_caches = Vec::with_capacity(*ec);
        for _ in 0..*ec {
            edge_caches.push(<C::Selector as DistanceSelector>::EdgeCache::default());
        }
        Self {
            shape,
            contour_combiner: cc,
            edge_cache: edge_caches,
        }
    }
    pub fn distance(
        &mut self,
        origin: <C::Selector as DistanceSelector>::ResetType,
    ) -> <C::Selector as DistanceSelector>::DistanceType {
        self.contour_combiner.reset(origin);
        let mut edge_cache_iter = self.edge_cache.iter_mut();
        for (contour_index, contour) in self.shape.contours.iter().enumerate() {
            if contour.edges.is_empty() {
                continue;
            }
            let edge_selector = self.contour_combiner.edge_selector(contour_index);
            let mut prev_edge = if contour.edges.len() >= 2 {
                contour.edges[contour.edges.len() - 2]
            } else {
                contour.edges[0]
            };
            let mut cur_edge = *contour.edges.last().unwrap();
            for &next_edge in contour.edges.iter() {
                let cache = edge_cache_iter.next().expect("shape_edge_cache too small");
                edge_selector.add_edge(cache, prev_edge, cur_edge, next_edge);
                prev_edge = cur_edge;
                cur_edge = next_edge;
            }
        }
        self.contour_combiner.distance()
    }
    pub fn one_shot_distance(&mut self, origin: <C::Selector as DistanceSelector>::ResetType) {}
}
