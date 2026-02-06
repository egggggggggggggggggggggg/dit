use std::ops::{Add, Div, Mul, Neg, Sub};

use math::bezier::Bounds;

pub struct Padding {
    l: f64,
    b: f64,
    r: f64,
    t: f64,
}
//rewrtiw
pub fn pad(bounds: &mut Bounds, padding: Padding) {
    bounds.l -= padding.l;
    bounds.b -= padding.b;
    bounds.r += padding.r;
    bounds.t += padding.t;
}
impl Add for Padding {
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            l: self.l + rhs.l,
            b: self.b + rhs.b,
            r: self.r + rhs.r,
            t: self.t + rhs.t,
        }
    }
    type Output = Self;
}
impl Sub for Padding {
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            l: self.l - rhs.l,
            b: self.b - rhs.b,
            r: self.r - rhs.r,
            t: self.t - rhs.t,
        }
    }
    type Output = Self;
}
impl Neg for Padding {
    fn neg(self) -> Self::Output {
        Self {
            l: -self.l,
            b: -self.b,
            r: -self.r,
            t: -self.t,
        }
    }
    type Output = Self;
}
impl Mul<f64> for Padding {
    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            l: self.l * rhs,
            b: self.b * rhs,
            r: self.r * rhs,
            t: self.t * rhs,
        }
    }
    type Output = Self;
}
impl Mul<Padding> for f64 {
    fn mul(self, rhs: Padding) -> Self::Output {
        Padding {
            l: self * rhs.l,
            b: self * rhs.b,
            r: self * rhs.r,
            t: self * rhs.t,
        }
    }
    type Output = Padding;
}
impl Div<f64> for Padding {
    fn div(self, rhs: f64) -> Self::Output {
        Padding {
            l: self.l / rhs,
            b: self.b / rhs,
            r: self.r / rhs,
            t: self.t / rhs,
        }
    }
    type Output = Self;
}
