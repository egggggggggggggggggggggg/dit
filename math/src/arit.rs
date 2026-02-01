use crate::lalg::Vec2;

#[inline(always)]
pub fn non_zero_sign(n: f64) -> f64 {
    2.0 * ((n > 0.0) as u8 as f64) - 1.0
}
#[inline(always)]
pub fn sign(n: f64) -> f64 {
    ((0.0 < n) as u8 as f64) - ((n < 0.0) as u8 as f64)
}
#[inline(always)]
pub fn mix(a: Vec2, b: Vec2, weight: f64) -> Vec2 {
    a * (1.0 - weight) + b * weight
}
#[inline(always)]
pub fn mixf(a: f64, b: f64, weight: f64) -> f64 {
    a * (1.0 - weight) + b * weight
}
#[inline(always)]
pub fn clamp() {}
#[inline(always)]
pub fn clamp_b() {}

#[inline(always)]
pub fn shoelace(a: Vec2, b: Vec2) -> f64 {
    (b.x - a.x) * (a.y + b.y)
}
