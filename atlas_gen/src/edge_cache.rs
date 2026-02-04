use math::lalg::Vec2;

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
#[derive(Debug, Clone, Default)]
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
