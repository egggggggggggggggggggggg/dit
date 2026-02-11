use std::{
    cmp::Ordering,
    ops::{Add, Div, Mul, Neg, Sub},
};

use crate::bezier::{Bezier, BezierTypes, CubicBezier, EdgeColor, LinearBezier, QuadraticBezier};
#[derive(Debug, Copy, Clone, Default)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}
impl Vec2 {
    #[inline(always)]
    pub fn magnitude(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
    #[inline(always)]
    pub fn dot(&self, rhs: Self) -> f64 {
        self.x * rhs.x + self.y * rhs.y
    }
    #[inline(always)]
    pub fn cross(&self, rhs: Self) -> f64 {
        self.x * rhs.y - self.y * rhs.x
    }
    #[inline(always)]
    pub fn normalize(&self) -> Self {
        self.normalize_allow_zero(false)
    }
    #[inline(always)]
    pub fn normalize_allow_zero(&self, allow_zero: bool) -> Self {
        let len = self.length();

        if len != 0.0 {
            Self {
                x: self.x / len,
                y: self.y / len,
            }
        } else if allow_zero {
            Self { x: 0.0, y: 0.0 }
        } else {
            Self { x: 0.0, y: 1.0 }
        }
    }
    #[inline(always)]
    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    #[inline(always)]
    pub fn squared_length(&self) -> f64 {
        self.x * self.x + self.y * self.y
    }
    #[inline(always)]
    pub fn orthogonal(&self, polarity: bool) -> Vec2 {
        if polarity {
            Vec2 {
                x: -self.y,
                y: -self.x,
            }
        } else {
            Vec2 {
                x: self.y,
                y: -self.x,
            }
        }
    }
    #[inline(always)]
    pub fn orthonormal(&self, polarity: bool, allow_zero: bool) -> Vec2 {
        let len = self.length();
        let sign = if polarity { 1.0 } else { -1.0 };

        if len != 0.0 {
            Vec2 {
                x: -sign * self.y / len,
                y: sign * self.x / len,
            }
        } else {
            Vec2 {
                x: 0.0,
                y: sign * (allow_zero as u8 as f64),
            }
        }
    }
    #[inline(always)]
    pub fn is_zero(&self) -> bool {
        self.x != 0.0 && self.y != 0.0
    }
}

impl Mul<f64> for Vec2 {
    fn mul(self, rhs: f64) -> Self::Output {
        Vec2 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
    type Output = Self;
}
impl Div<f64> for Vec2 {
    fn div(self, rhs: f64) -> Self::Output {
        Vec2 {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
    type Output = Self;
}
impl Sub for Vec2 {
    fn sub(self, rhs: Self) -> Self::Output {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
    type Output = Self;
}
impl Add for Vec2 {
    fn add(self, rhs: Self) -> Self::Output {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
    type Output = Self;
}
//dot product
impl Mul for Vec2 {
    fn mul(self, rhs: Self) -> Self::Output {
        (self.x * rhs.x) + (self.y * rhs.y)
    }
    type Output = f64;
}
impl Neg for Vec2 {
    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
    type Output = Self;
}
impl Eq for Vec2 {}
impl PartialEq for Vec2 {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub dx: f64,
    pub dy: f64,
}
impl Transform {
    pub fn identity() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            dx: 0.0,
            dy: 0.0,
        }
    }
    #[inline(always)]
    pub fn apply(&self, p: Vec2) -> Vec2 {
        Vec2 {
            x: self.a * p.x + self.b * p.y + self.dx,
            y: self.c * p.x + self.d * p.y + self.dy,
        }
    }
    #[inline(always)]
    pub fn combine(self, other: Transform) -> Transform {
        Transform {
            a: self.a * other.a + self.b * other.c,
            b: self.a * other.b + self.b * other.d,
            c: self.c * other.a + self.d * other.c,
            d: self.c * other.b + self.d * other.d,
            dx: self.a * other.dx + self.b * other.dy + self.dx,
            dy: self.c * other.dx + self.d * other.dy + self.dy,
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub enum BezierCurve {
    Linear(Vec2, Vec2),
    Quadratic(Vec2, Vec2, Vec2),
    Cubic(Vec2, Vec2, Vec2, Vec2),
}
pub fn transform_curve(curve: &BezierTypes, t: Transform) -> BezierTypes {
    match *curve {
        BezierTypes::Linear(l_bezier) => BezierTypes::Linear(LinearBezier::new(
            t.apply(l_bezier.p[0]),
            t.apply(l_bezier.p[1]),
            EdgeColor::WHITE,
        )),
        BezierTypes::Quadratic(q_bezier) => BezierTypes::Quadratic(QuadraticBezier::new(
            t.apply(q_bezier.p[0]),
            t.apply(q_bezier.p[1]),
            t.apply(q_bezier.p[2]),
            EdgeColor::WHITE,
        )),
        BezierTypes::Cubic(c_bezier) => BezierTypes::Cubic(CubicBezier::new(
            t.apply(c_bezier.p[0]),
            t.apply(c_bezier.p[1]),
            t.apply(c_bezier.p[2]),
            t.apply(c_bezier.p[3]),
            EdgeColor::WHITE,
        )),
    }
}
impl BezierCurve {
    pub fn evaluate_bezier(&self, t: f64) -> Vec2 {
        let u = 1.0 - t;
        match self {
            BezierCurve::Cubic(_a, _b, _c, _d) => Vec2 { x: 0.0, y: 0.0 },
            BezierCurve::Quadratic(p0, p1, p2) => {
                *p0 * (u * u) + *p1 * (2.0 * u * t) + *p2 * (t * t)
            }
            BezierCurve::Linear(p0, p1) => (*p0 * u) + (*p1 * t),
        }
    }
    pub fn derive_curve(&self) -> BezierCurve {
        match self {
            BezierCurve::Linear(p0, p1) => {
                // Derivative of linear Bézier: constant vector (p1 - p0)
                BezierCurve::Linear(*p1 - *p0, *p1 - *p0)
            }
            BezierCurve::Quadratic(p0, p1, p2) => {
                // Derivative of quadratic Bézier: 2 * (1 - t) * (p1 - p0) + 2 * t * (p2 - p1)
                BezierCurve::Linear((*p1 - *p0) * 2.0, (*p2 - *p1) * 2.0)
            }
            BezierCurve::Cubic(p0, p1, p2, p3) => {
                // Derivative of cubic Bézier: 3 * (1 - t)^2 * (p1 - p0) + 6 * (1 - t) * t * (p2 - p1) + 3 * t^2 * (p3 - p2)
                BezierCurve::Quadratic((*p1 - *p0) * 3.0, (*p2 - *p1) * 6.0, (*p3 - *p2) * 3.0)
            }
        }
    }
    pub fn split_in_thirds(&mut self) {}
}
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct BinaryVector {
    pub x: bool,
    pub y: bool,
    pub z: bool,
}

impl BinaryVector {
    pub const BLACK: Self = Self {
        x: false,
        y: false,
        z: false,
    };
    pub const RED: Self = Self {
        x: true,
        y: false,
        z: false,
    };
    pub const GREEN: Self = Self {
        x: false,
        y: true,
        z: false,
    };
    pub const BLUE: Self = Self {
        x: false,
        y: false,
        z: true,
    };
    pub const YELLOW: Self = Self {
        x: true,
        y: true,
        z: false,
    };
    pub const MAGENTA: Self = Self {
        x: true,
        y: false,
        z: true,
    };
    pub const CYAN: Self = Self {
        x: false,
        y: true,
        z: true,
    };
    pub const WHITE: Self = Self {
        x: true,
        y: true,
        z: true,
    };
    pub fn new(x: bool, y: bool, z: bool) -> Self {
        Self { x, y, z }
    }
    pub fn dot(&self, rhs: &Self) -> u8 {
        (self.x & rhs.x) as u8 + (self.y & rhs.y) as u8 + (self.z & rhs.z) as u8
    }
    pub fn median(&self) -> bool {
        ((self.x & self.y) | (self.x & self.z) | (self.y & self.z))
    }
}
