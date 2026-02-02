use math::{
    bezier::{Bezier, BezierTypes, SignedDistance},
    lalg::Vec2,
};

const DISTANCE_DELTA_FACTOR: f64 = 1.001;
#[derive(Debug, Clone)]
pub struct TrueDistanceEdgeCache {
    pub point: Vec2,
    pub abs_distance: f64,
}

impl TrueDistanceEdgeCache {
    pub fn new() -> Self {
        Self {
            point: Vec2::default(),
            abs_distance: f64::INFINITY,
        }
    }
    pub fn reset(&mut self) {
        self.abs_distance = f64::INFINITY;
    }
}
#[derive(Debug, Clone)]
pub struct PerpendicularEdgeCache {
    pub point: Vec2,
    pub abs_distance: f64,
    pub a_domain_distance: f64,
    pub b_domain_distance: f64,
    pub a_perpendicular_distance: f64,
    pub b_perpendicular_distance: f64,
}
impl PerpendicularEdgeCache {
    pub fn new() -> Self {
        Self {
            point: Vec2::default(),
            abs_distance: f64::INFINITY,
            a_domain_distance: f64::INFINITY,
            b_domain_distance: f64::INFINITY,
            a_perpendicular_distance: 0.0,
            b_perpendicular_distance: 0.0,
        }
    }
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}
#[derive(Clone)]
struct PerpendicularDistanceSelector {}
impl PerpendicularDistanceSelector {}

trait DistanceSelector {
    type Distance;
    type EdgeCache;
    fn reset(&mut self, p: Vec2);
    fn add_edge(
        &mut self,
        cache: Self::EdgeCache,
        prev_edge: BezierTypes,
        edge: BezierTypes,
        next_edge: BezierTypes,
    );
    fn merge(&mut self, other: Self);
    fn distance(&mut self) -> Self::Distance;
    fn true_distance() -> SignedDistance;
}
#[derive(Clone)]
struct MultiDistanceSelector {
    p: Vec2,
    r: PerpendicularDistanceSelector,
    g: PerpendicularDistanceSelector,
    b: PerpendicularDistanceSelector,
}
