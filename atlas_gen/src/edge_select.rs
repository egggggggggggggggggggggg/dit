use core::f64;

use crate::{
    distances::{DistanceType, MultiDistance, RegDistance},
    edge_cache::{PerpendicularEdgeCache, TrueDistanceEdgeCache},
};
use math::{
    arit::non_zero_sign,
    bezier::{Bezier, BezierTypes, EdgeColor, SignedDistance},
    lalg::Vec2,
};
const DISTANCE_DELTA_FACTOR: f64 = 1.001;

pub trait DistanceSelector: Default {
    type ResetType;
    type DistanceType: DistanceType;
    type EdgeCache;
    fn reset(&mut self, d: Self::ResetType);
    fn add_edge(
        &mut self,
        cache: &mut Self::EdgeCache,
        prev_edge: BezierTypes,
        edge: BezierTypes,
        next_edge: BezierTypes,
    );
    fn merge(&mut self, other: &Self);
    fn distance(&mut self) -> Self::DistanceType;
    fn new() -> Self;
}

#[derive(Default, Clone)]
pub struct PerpDistSelectorBase {
    edge_cache: PerpendicularEdgeCache,
    min_true_distance: SignedDistance,
    min_neg_perp_distance: f64,
    min_pos_perp_distance: f64,
    near_edge: Option<BezierTypes>,
    near_edge_param: f64,
}
impl PerpDistSelectorBase {
    pub fn new() -> Self {
        Self {
            min_true_distance: SignedDistance::default(),
            min_neg_perp_distance: -f64::MAX,
            min_pos_perp_distance: f64::MAX,
            edge_cache: PerpendicularEdgeCache::default(),
            near_edge: None,
            near_edge_param: 0.0,
        }
    }
    pub fn reset(&mut self, delta: f64) {
        self.min_true_distance.distance += non_zero_sign(self.min_true_distance.distance) * delta;
        self.min_neg_perp_distance = -self.min_true_distance.distance.abs();
        self.min_pos_perp_distance = self.min_true_distance.distance.abs();
        self.near_edge = None;
        self.near_edge_param = 0.0;
    }
    pub fn is_edge_relevant(
        &self,
        cache: &PerpendicularEdgeCache,
        edge: BezierTypes,
        p: Vec2,
    ) -> bool {
        //might be wrong
        let delta = DISTANCE_DELTA_FACTOR * (p - cache.point).length();

        let r0 = cache.abs_distance - delta <= self.min_true_distance.distance.abs();
        let r1 = cache.a_domain_distance.abs() < delta;
        let r2 = cache.b_domain_distance.abs() < delta;

        let r3 = cache.a_domain_distance > 0.0;
        let r4 = cache.a_perpendicular_distance < 0.0;
        let r5 = cache.a_perpendicular_distance + delta >= self.min_neg_perp_distance;
        let r6 = cache.a_perpendicular_distance - delta <= self.min_pos_perp_distance;

        let r7 = cache.b_domain_distance > 0.0;
        let r8 = cache.b_perpendicular_distance < 0.0;
        let r9 = cache.b_perpendicular_distance + delta >= self.min_neg_perp_distance;
        let r10 = cache.b_perpendicular_distance - delta <= self.min_pos_perp_distance;

        let conditions = r0
            || r1
            || r2
            || (r3 && ((r4 && r5) || (!r4 && r6)))
            || (r7 && ((r8 && r9) || (!r8 && r10)));
        conditions
    }
    pub fn add_edge_true_distance(
        &mut self,
        edge: BezierTypes,
        distance: SignedDistance,
        param: f64,
    ) {
        if distance < self.min_true_distance {
            self.min_true_distance = distance;
            self.near_edge = Some(edge);
            self.near_edge_param = param;
        }
    }
    pub fn add_edge_perpendicular_distance(&mut self, distance: f64) {
        if distance <= 0.0 && distance > self.min_neg_perp_distance {
            self.min_neg_perp_distance = distance;
        }
        if distance >= 0.0 && distance < self.min_pos_perp_distance {
            self.min_pos_perp_distance = distance;
        }
    }
    pub fn merge(&mut self, other: &Self) {
        if other.min_true_distance < self.min_true_distance {
            self.min_true_distance = other.min_true_distance;
            self.near_edge = other.near_edge;
            self.near_edge_param = other.near_edge_param;
        }
        if other.min_neg_perp_distance > self.min_neg_perp_distance {
            self.min_neg_perp_distance = other.min_neg_perp_distance;
        }
        if other.min_pos_perp_distance < self.min_pos_perp_distance {
            self.min_pos_perp_distance = other.min_pos_perp_distance;
        }
    }
    pub fn compute_distance(&mut self, p: Vec2) -> f64 {
        let mut min_dist = if self.min_true_distance.distance < 0.0 {
            self.min_neg_perp_distance
        } else {
            self.min_pos_perp_distance
        };
        if let Some(near_edge) = self.near_edge {
            let mut distance = self.min_true_distance;
            near_edge.distance_to_perpendicular_distance(&mut distance, p, self.near_edge_param);
            if distance.distance.abs() < min_dist.abs() {
                min_dist = distance.distance;
            }
        }
        min_dist
    }
    pub fn true_distance(&self) -> SignedDistance {
        self.min_true_distance
    }
}

#[derive(Default, Clone)]
pub struct PerpendicularDistanceSelector {
    base: PerpDistSelectorBase,
    p: Vec2,
}
impl DistanceSelector for PerpendicularDistanceSelector {
    type DistanceType = RegDistance;
    type EdgeCache = PerpendicularEdgeCache;
    type ResetType = Vec2;
    fn new() -> Self {
        Self {
            base: PerpDistSelectorBase::new(),
            p: Vec2::default(),
        }
    }
    fn add_edge(
        &mut self,
        cache: &mut Self::EdgeCache,
        prev_edge: BezierTypes,
        edge: BezierTypes,
        next_edge: BezierTypes,
    ) {
        if self.base.is_edge_relevant(&cache, edge, self.p) {
            let mut param = 0.0;
            let distance = edge.signed_distance(self.p, &mut param);
            self.base.add_edge_true_distance(edge, distance, param);
            cache.point = self.p;
            cache.abs_distance = distance.distance.abs();
            let ap = self.p - edge.point(0.0);
            let bp = self.p - edge.point(1.0);
            let a_dir = edge.direction(0.0).normalize_allow_zero(true);
            let b_dir = edge.direction(1.0).normalize_allow_zero(true);
            let prev_dir = prev_edge.direction(1.0).normalize_allow_zero(true);
            let next_dir = next_edge.direction(0.0).normalize_allow_zero(true);
            let add = ap.dot((prev_dir + a_dir).normalize_allow_zero(true));
            let bdd = -bp.dot((b_dir + next_dir).normalize_allow_zero(true));
            if add > 0.0 {
                let mut pd = distance.distance;
                if get_perpendicular_distance(&mut pd, ap, -a_dir) {
                    pd = -pd;
                    self.base.add_edge_perpendicular_distance(pd);
                }
                cache.a_perpendicular_distance = pd;
            }
            if bdd > 0.0 {
                let mut pd = distance.distance;
                if get_perpendicular_distance(&mut pd, bp, b_dir) {
                    self.base.add_edge_perpendicular_distance(pd);
                }
                cache.b_perpendicular_distance = pd;
            }
            cache.a_domain_distance = add;
            cache.b_domain_distance = bdd;
        }
    }
    fn reset(&mut self, d: Self::ResetType) {
        let delta = DISTANCE_DELTA_FACTOR * (d - self.p).length();
        self.base.reset(delta);
        self.p = d;
    }
    fn distance(&mut self) -> Self::DistanceType {
        self.base.compute_distance(self.p)
    }
    fn merge(&mut self, other: &Self) {
        self.base.merge(&other.base);
    }
}

#[derive(Default, Clone)]
pub struct MultiDistanceSelector {
    p: Vec2,
    r: PerpDistSelectorBase,
    g: PerpDistSelectorBase,
    b: PerpDistSelectorBase,
}
impl DistanceSelector for MultiDistanceSelector {
    type DistanceType = MultiDistance;
    type EdgeCache = PerpendicularEdgeCache;
    type ResetType = Vec2;
    fn new() -> Self {
        Self::default()
    }
    fn add_edge(
        &mut self,
        cache: &mut Self::EdgeCache,
        prev_edge: BezierTypes,
        edge: BezierTypes,
        next_edge: BezierTypes,
    ) {
        let contains_red = edge.color().contains(EdgeColor::RED);
        let contains_green = edge.color().contains(EdgeColor::GREEN);
        let contains_blue = edge.color().contains(EdgeColor::BLUE);
        if (edge.color().contains(EdgeColor::RED) && self.r.is_edge_relevant(&cache, edge, self.p))
            || (edge.color().contains(EdgeColor::GREEN)
                && self.g.is_edge_relevant(&cache, edge, self.p))
            || (edge.color().contains(EdgeColor::BLUE)
                && self.b.is_edge_relevant(&cache, edge, self.p))
        {
            let mut param = 0.0;
            let distance = edge.signed_distance(self.p, &mut param);
            if contains_red {
                self.r.add_edge_true_distance(edge, distance, param);
            }
            if contains_green {
                self.g.add_edge_true_distance(edge, distance, param);
            }
            if contains_blue {
                self.b.add_edge_true_distance(edge, distance, param);
            }
            cache.point = self.p;
            cache.abs_distance = distance.distance.abs();

            let ap = self.p - edge.point(0.0);
            let bp = self.p - edge.point(1.0);
            let a_dir = edge.direction(0.0).normalize_allow_zero(true);
            let b_dir = edge.direction(1.0).normalize_allow_zero(true);
            let prev_dir = prev_edge.direction(1.0).normalize_allow_zero(true);
            let next_dir = next_edge.direction(0.0).normalize_allow_zero(true);
            let add = ap.dot((prev_dir + a_dir).normalize_allow_zero(true));
            let bdd = -bp.dot((b_dir + next_dir).normalize_allow_zero(true));
            if add > 0.0 {
                let mut pd = distance.distance;
                if get_perpendicular_distance(&mut pd, ap, -a_dir) {
                    pd = -pd;
                    if contains_red {
                        self.r.add_edge_perpendicular_distance(pd);
                    }
                    if contains_green {
                        self.g.add_edge_perpendicular_distance(pd);
                    }
                    if contains_blue {
                        self.b.add_edge_perpendicular_distance(pd);
                    }
                }
                cache.a_perpendicular_distance = pd;
            }
            if bdd > 0.0 {
                let mut pd = distance.distance;
                if get_perpendicular_distance(&mut pd, ap, b_dir) {
                    if contains_red {
                        self.r.add_edge_perpendicular_distance(pd);
                    }
                    if contains_green {
                        self.g.add_edge_perpendicular_distance(pd);
                    }
                    if contains_blue {
                        self.b.add_edge_perpendicular_distance(pd);
                    }
                }
                cache.a_perpendicular_distance = pd;
            }
            cache.a_domain_distance = add;
            cache.b_domain_distance = bdd;
        }
    }
    fn distance(&mut self) -> Self::DistanceType {
        let mut multi_distance = MultiDistance::default();
        multi_distance.r = self.r.compute_distance(self.p);
        multi_distance.g = self.g.compute_distance(self.p);
        multi_distance.b = self.b.compute_distance(self.p);
        multi_distance
    }
    fn merge(&mut self, other: &Self) {
        self.r.merge(&other.r);
        self.g.merge(&other.g);
        self.b.merge(&other.b);
    }
    fn reset(&mut self, d: Self::ResetType) {
        let delta = DISTANCE_DELTA_FACTOR * (d - self.p).length();
        self.r.reset(delta);
        self.g.reset(delta);
        self.b.reset(delta);
        self.p = d;
    }
}

#[derive(Default, Clone)]
pub struct TrueDistanceSelector {
    p: Vec2,
    min_distance: SignedDistance,
}
impl DistanceSelector for TrueDistanceSelector {
    type DistanceType = RegDistance;
    type EdgeCache = TrueDistanceEdgeCache;
    type ResetType = Vec2;
    fn new() -> Self {
        Self::default()
    }
    fn add_edge(
        &mut self,
        cache: &mut Self::EdgeCache,
        prev_edge: BezierTypes,
        edge: BezierTypes,
        next_edge: BezierTypes,
    ) {
        let delta = DISTANCE_DELTA_FACTOR * (self.p - cache.point).length();
        if cache.abs_distance - delta <= self.min_distance.distance.abs() {
            let mut dummy = 0.0;
            let distance = edge.signed_distance(self.p, &mut dummy);
            if distance < self.min_distance {
                self.min_distance = distance;
            }
            cache.point = self.p;
            cache.abs_distance = distance.distance.abs();
        }
    }
    fn distance(&mut self) -> Self::DistanceType {
        self.min_distance.distance
    }
    fn merge(&mut self, other: &Self) {
        if other.min_distance < self.min_distance {
            self.min_distance = other.min_distance;
        }
    }
    fn reset(&mut self, d: Self::ResetType) {
        let delta = (d - self.p).length() * DISTANCE_DELTA_FACTOR;
        self.min_distance.distance += non_zero_sign(self.min_distance.distance) * delta;
        self.p = d;
    }
}
pub fn get_perpendicular_distance(distance: &mut f64, ep: Vec2, edge_dir: Vec2) -> bool {
    let ts = ep.dot(edge_dir);
    if ts > 0.0 {
        let perpendicular_distance = ep.cross(edge_dir);
        if perpendicular_distance.abs() < distance.abs() {
            *distance = perpendicular_distance;
            return true;
        }
    }
    false
}
