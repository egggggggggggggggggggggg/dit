use std::{
    iter::Sum,
    ops::{Add, Div, Mul, Sub},
};

use image::{ImageBuffer, Rgb};
fn main() {
    println!("Hello, world!");
    let p0 = Vec2 { x: 0.0, y: 0.0 };
    let p1 = Vec2 { x: 256.0, y: 256.0 };
    let p2 = Vec2 { x: 512.0, y: 512.0 };
    let qbc = QBezierCurve {
        points: [p0, p1, p2],
    };
    let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(512, 512);
    generate_sdf(qbc, &mut img);
    draw_curve(qbc.clone(), 1000);
}
struct Position {
    x: f32,
    y: f32,
}
#[derive(Clone, Copy)]
struct QBezierCurve {
    points: [Vec2; 3],
}
fn plot(img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, x: i32, y: i32, intensity: f32) {
    if x < 0 || y < 0 || x >= img.width() as i32 || y >= img.height() as i32 {
        return;
    }
    let pixel = img.get_pixel_mut(x as u32, y as u32);
    let r = pixel[0] as f32 + 255.0 * intensity;
    let g = pixel[1] as f32 + 255.0 * intensity;
    let b = pixel[2] as f32 + 128.0 * intensity;

    *pixel = Rgb([r.min(255.0) as u8, g.min(0.0) as u8, b.min(0.0) as u8]);
}

fn draw_curve(curve: QBezierCurve, sample_size: u32) {
    let points = curve.points;
    let width = 512;
    let height = 512;
    let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);
    for i in 0..sample_size {
        let t = i as f32 / sample_size as f32;
        let coef = 1.0 - t as f32;
        let x = coef.powi(2) * points[0].x + 2.0 * coef * t * points[1].x + t.powi(2) * points[2].x;
        let y = coef.powi(2) * points[0].y + 2.0 * coef * t * points[1].y + t.powi(2) * points[2].y;
        //get the fractional and whole component for anti-aliasing
        let x0 = x.floor();
        let y0 = y.floor();

        let fx = x - x0;
        let fy = y - y0;

        plot(&mut img, x0 as i32, y0 as i32, (1.0 - fx) * (1.0 - fy));
        plot(&mut img, (x0 + 1.0) as i32, y0 as i32, fx * (1.0 - fy));
        plot(&mut img, x0 as i32, (y0 + 1.0) as i32, (1.0 - fx) * fy);
        plot(&mut img, (x0 + 1.0) as i32, (y0 + 1.0) as i32, fx * fy);
    }

    img.save("output.png").unwrap();
}
fn generate_msdf() {}
enum QBCl {
    Curve(QBezierCurve),
    Line(QBezierCurve),
}
fn is_linear(curve: &QBezierCurve) -> bool {
    //this doesn't account for colinearity which is what actually determines if a
    //bezier curve is linear
    let p0 = curve.points[0];
    let p1 = curve.points[1];
    let p2 = curve.points[2];
    let slope_01 = (p1.y - p0.y) / (p1.x - p0.x);
    let slope_12 = (p2.y - p1.y) / (p2.x - p1.x);
    if slope_01 == slope_12 {
        true
    }
    false
}

fn generate_sdf(curve: QBezierCurve, img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) {
    //first generalize the curve
    //operates on a single curve qw
    let p0 = curve.points[0];
    let p1 = curve.points[1];
    let p2 = curve.points[2];
    if is_linear(&curve) {
        //known solution for a straight line
        for x in 0..512 {
            for y in 0..512 {
                let pos = Vec2 {
                    x: x as f32,
                    y: y as f32,
                };
                let t = ((pos - p0) * (p2 - p0)) / (p2 - p0).magnitude().powi(2);
                let clamped_t = t.min(1.0).max(0.0);
                let c = p0 + (p2 - p0) * clamped_t;
                println!("{}", (pos - c).magnitude());

                plot(img, x, y, (pos - c).magnitude() / 512.0);
            }
        }
    } else {
    }
    img.save("output2.png").unwrap();
}

trait Vector {}
#[derive(Copy, Clone)]
struct Vec2 {
    x: f32,
    y: f32,
}
impl Vec2 {
    fn magnitude(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
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
impl Mul for Vec2 {
    fn mul(self, rhs: Self) -> Self::Output {
        (self.x * rhs.x) + (self.y + rhs.y)
    }
    type Output = f32;
}

//problem trying to solve :
//find the distance of a given point from a contour/line segment
//this distance must have a sign value indicating inside or outside of the shape
//to do so you must find the min distance for this.
//this is classic sdf
//msdf solves an issue with classic sdf which is the issue of edge bleeding
//by having multipl distance values for different contours we can have more accurate contours
//to solve for the min distance of a cubic bezier curve from a given point, solve its derirative
//t is the value on the curve that yields the min dist from a given point to the curve
//some form of ray marching or analytically solve the polynomial
//
