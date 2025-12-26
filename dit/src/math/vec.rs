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
