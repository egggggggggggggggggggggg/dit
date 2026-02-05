use std::ops::{Div, DivAssign, Mul, MulAssign};

use math::{calc::median, lalg::Vec2};

//workaround for types being unable to defined in an impl block
//could possible write a macro that allows you to access all the fields of a struct or smth
pub trait DistanceType: Default {
    fn resolve(&self) -> f64;
    fn init() -> Self;
}
pub type RegDistance = f64;
impl DistanceType for RegDistance {
    fn resolve(&self) -> f64 {
        *self
    }
    fn init() -> Self {
        -f64::MAX
    }
}

#[derive(Clone, Default, Debug)]
pub struct MultiDistance {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}
impl MultiDistance {
    fn new() -> Self {
        Self {
            r: -f64::MAX,
            b: -f64::MAX,
            g: -f64::MAX,
        }
    }
}
impl DistanceType for MultiDistance {
    fn resolve(&self) -> f64 {
        median(self.r, self.g, self.b)
    }
    fn init() -> Self {
        Self::new()
    }
}
#[derive(Clone, Default, Debug)]
pub struct MultiAndTrueDistance {
    pub base: MultiDistance,
    pub a: f64,
}
impl MultiAndTrueDistance {
    fn new() -> Self {
        Self {
            base: MultiDistance::new(),
            a: -f64::MAX,
        }
    }
}
impl DistanceType for MultiAndTrueDistance {
    fn resolve(&self) -> f64 {
        median(self.base.r, self.base.g, self.base.b)
    }
    fn init() -> Self {
        Self::new()
    }
}
struct DistanceMapping {
    scale: f64,
    translate: f64,
}
impl Default for DistanceMapping {
    fn default() -> Self {
        Self {
            scale: 1.0,
            translate: 0.0,
        }
    }
}
impl DistanceMapping {
    pub fn new(scale: f64, translate: f64) -> Self {
        Self { scale, translate }
    }
    pub fn inverse_with_range(range: Range) -> Self {
        let range_width = range.upper - range.lower;
        let translate = range.lower / if range_width != 0.0 { range_width } else { 1.0 };
        Self::new(range_width, translate)
    }
    pub fn inverse(&self) -> Self {
        Self {
            scale: 1.0 / self.scale,
            translate: -self.scale * self.translate,
        }
    }
    pub fn apply_with_transform(&self, d: f64) -> f64 {
        self.scale * (d + self.translate)
    }
    pub fn apply(&self, d: f64) -> f64 {
        self.scale * d
    }
}

struct Range {
    lower: f64,
    upper: f64,
}
impl Range {
    #[inline(always)]
    fn symmetrical(width: f64) -> Self {
        Self {
            lower: -0.5 * width,
            upper: 0.5 * width,
        }
    }
    #[inline(always)]
    fn bounds(lower: f64, upper: f64) -> Self {
        Self { upper, lower }
    }
}
impl MulAssign<f64> for Range {
    fn mul_assign(&mut self, rhs: f64) {
        self.lower *= rhs;
        self.upper *= rhs;
    }
}
impl DivAssign<f64> for Range {
    fn div_assign(&mut self, rhs: f64) {
        self.lower /= rhs;
        self.upper /= rhs;
    }
}
impl Mul<f64> for Range {
    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            lower: self.lower * rhs,
            upper: self.upper * rhs,
        }
    }
    type Output = Self;
}
impl Div<f64> for Range {
    fn div(self, rhs: f64) -> Self::Output {
        Self {
            lower: self.lower / rhs,
            upper: self.upper / rhs,
        }
    }
    type Output = Self;
}
impl Mul<Range> for f64 {
    type Output = Range;
    fn mul(self, rhs: Range) -> Self::Output {
        Range {
            lower: self * rhs.lower,
            upper: self * rhs.upper,
        }
    }
}
trait DistancePixelConversion {
    type Distance;
    const CHANNELS: usize;
    fn write_pixel(&self, pixels: &mut [f32], distance: Self::Distance);
}

#[derive(Copy, Clone, Debug)]
pub struct Projection {
    pub scale: Vec2,
    pub translate: Vec2,
}

impl Default for Projection {
    fn default() -> Self {
        Self {
            scale: Vec2 { x: 1.0, y: 1.0 },
            translate: Vec2 { x: 0.0, y: 0.0 },
        }
    }
}

impl Projection {
    pub fn new(scale: Vec2, translate: Vec2) -> Self {
        Self { scale, translate }
    }

    pub fn project(&self, coord: Vec2) -> Vec2 {
        Vec2 {
            x: self.scale.x * (coord.x + self.translate.x),
            y: self.scale.y * (coord.y + self.translate.y),
        }
    }

    pub fn unproject(&self, coord: Vec2) -> Vec2 {
        Vec2 {
            x: coord.x / self.scale.x - self.translate.x,
            y: coord.y / self.scale.y - self.translate.y,
        }
    }

    pub fn project_vector(&self, vector: Vec2) -> Vec2 {
        Vec2 {
            x: self.scale.x * vector.x,
            y: self.scale.y * vector.y,
        }
    }

    pub fn unproject_vector(&self, vector: Vec2) -> Vec2 {
        Vec2 {
            x: vector.x / self.scale.x,
            y: vector.y / self.scale.y,
        }
    }

    pub fn project_x(&self, x: f64) -> f64 {
        self.scale.x * (x + self.translate.x)
    }

    pub fn project_y(&self, y: f64) -> f64 {
        self.scale.y * (y + self.translate.y)
    }

    pub fn unproject_x(&self, x: f64) -> f64 {
        x / self.scale.x - self.translate.x
    }

    pub fn unproject_y(&self, y: f64) -> f64 {
        y / self.scale.y - self.translate.y
    }
}
