use std::ops::{Add, Div, Mul, Sub};
#[derive(Debug, Copy, Clone)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}
impl Vec2 {
    pub fn magnitude(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
    pub fn dot(&self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y
    }
    pub fn cross(&self, rhs: Self) -> f32 {
        self.x * rhs.y - self.y * rhs.x
    }
    pub fn normalize(&self) -> Self {
        *self / self.magnitude()
    }
}

impl Mul<f32> for Vec2 {
    fn mul(self, rhs: f32) -> Self::Output {
        Vec2 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
    type Output = Self;
}
impl Div<f32> for Vec2 {
    fn div(self, rhs: f32) -> Self::Output {
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
    type Output = f32;
}

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub dx: f32,
    pub dy: f32,
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
pub fn transform_curve(curve: &BezierCurve, t: Transform) -> BezierCurve {
    match *curve {
        BezierCurve::Linear(p0, p1) => BezierCurve::Linear(t.apply(p0), t.apply(p1)),
        BezierCurve::Quadratic(p0, p1, p2) => {
            BezierCurve::Quadratic(t.apply(p0), t.apply(p1), t.apply(p2))
        }
        BezierCurve::Cubic(p0, p1, p2, p3) => {
            BezierCurve::Cubic(t.apply(p0), t.apply(p1), t.apply(p2), t.apply(p3))
        }
    }
}
impl BezierCurve {
    pub fn evaluate_bezier(&self, t: f32) -> Vec2 {
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
}
