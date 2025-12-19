use image::{ImageBuffer, Rgb};
use rand::prelude::*;
use std::{
    iter::Sum,
    ops::{Add, Div, Mul, Sub},
    thread::current,
};
const HEIGHT: u32 = 512;
const WIDTH: u32 = 512;
fn rand_vec2(xmax: f32, xmin: f32, ymax: f32, ymin: f32) -> Vec2 {
    //change this later to pass in rng instead to avoid creating and destroing rng gen
    let mut rng = rand::rng();
    Vec2 {
        x: rng.random_range(xmin..=xmax),
        y: rng.random_range(ymin..=ymax),
    }
}

fn main() {
    println!("Hello, world!");
    let p0 = Vec2 { x: 200.0, y: 200.0 };
    let p1 = Vec2 { x: 500.0, y: 100.0 };
    let p2 = Vec2 { x: 500.0, y: 450.0 };
    let qbc = QBezierCurve {
        points: [p0, p1, p2],
    };
    let p10 = Vec2 { x: 29.0, y: 41.0 };
    let p11 = Vec2 { x: 412.0, y: 512.0 };
    let p12 = Vec2 { x: 0.0, y: 200.0 };
    let qbc2 = QBezierCurve {
        points: [p10, p11, p12],
    };
    let curves = vec![qbc, qbc2];
    let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(512, 512);
    quadratic_case(curves, &mut img);
    draw_curve(qbc2.clone(), 1000);
}
const EPS64: f64 = 1e-9;
const ROOT_EPS64: f64 = 1e-7;
const EPS: f32 = 1e-6;
const ROOT_EPS: f32 = 1e-4;
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
fn plot_graph(img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, x: i32, y: i32, intensity: f32) {
    let width = img.width() as i32;
    let height = img.height() as i32;

    // Move origin to center and flip Y axis
    let cx = width / 2;
    let cy = height / 2;

    let img_x = cx + x;
    let img_y = cy - y;

    if img_x < 0 || img_y < 0 || img_x >= width || img_y >= height {
        return;
    }

    let pixel = img.get_pixel_mut(img_x as u32, img_y as u32);

    let r = pixel[0] as f32 + 255.0 * intensity;
    let g = pixel[1] as f32 + 255.0 * intensity;
    let b = pixel[2] as f32 + 128.0 * intensity;

    *pixel = Rgb([
        r.clamp(0.0, 255.0) as u8,
        g.clamp(0.0, 255.0) as u8,
        b.clamp(0.0, 255.0) as u8,
    ]);
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
        return true;
    }
    false
}
const SAMPLE_POINTS: usize = 80;
//for each point evaluate all the resulting bezier curves for a given glyph
//take that and run it thru a min_dist ranking algo
//take the top three and store those  in the rgb component such that the values are out of 255.0
//smth about padding to prevent bleeding
//
//rewrite this in glsl shader code
fn quadratic_case(curves: Vec<QBezierCurve>, img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) {
    let height = img.height();
    let width = img.width();
    for x in 0..width {
        for y in 0..height {
            let q = Vec2 {
                x: x as f32,
                y: y as f32,
            };
            let mut final_min = f32::MAX;
            for i in 0..curves.len() {
                let curve = &curves[i];
                let p0 = curve.points[0];
                let p1 = curve.points[1];
                let p2 = curve.points[2];
                let a = p0 - p1 * 2.0 + p2;
                let b = (p1 - p0) * 2.0;
                let c = p0;
                let k3 = (a * a) * 2.0;
                let k2 = (a * b) * 3.0;
                let l = c - q;
                let k1 = (b * b) + (a * (l * 2.0));
                let k0 = b * l;
                let cubic = Polynomial {
                    coefficients: vec![k3, k2, k1, k0],
                };
                let mut candidate_intervals: Vec<Range> = vec![];
                let mut i = 0;
                let mut roots = vec![0.0, 1.0];
                while i < SAMPLE_POINTS + 1 {
                    let first = cubic.eval_horny(i as f32 / SAMPLE_POINTS as f32);
                    let second = cubic.eval_horny((i + 1) as f32 / SAMPLE_POINTS as f32);
                    if (first.abs() < EPS)
                        || (second.abs() < EPS)
                        || (first.signum() != second.signum())
                    {
                        candidate_intervals.push(Range {
                            lower: i as f32 / SAMPLE_POINTS as f32,
                            higher: (i as f32 + 1.0) / SAMPLE_POINTS as f32,
                        })
                    }
                    i += 1;
                }
                for i in candidate_intervals {
                    if let Some(root) = bisection(&cubic, i) {
                        roots.push(root);
                    }
                }
                let min_dist = get_min_dist(p0, p1, p2, q, &roots);
                if min_dist < final_min {
                    final_min = min_dist;
                };
            }
            plot(img, x as i32, y as i32, final_min / 300.0);
        }
    }
    img.save("testIMG.png").unwrap();
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
                // a polynomial term can defined as polynomial struct with every other coefficient
                // set to 0  besides the term value itself,
                let t = ((pos - p0) * (p2 - p0)) / (p2 - p0).magnitude().powi(2);
                let clamped_t = t.min(1.0).max(0.0);
                let c = p0 + (p2 - p0) * clamped_t;
                println!("{}", (pos - c).magnitude());
                plot(img, x, y, (pos - c).magnitude() / 255.0);
            }
        }
    } else {
        //reduce the curve
        let a = p0 - p1 * 2.0 + p2;
        let b = (p1 - p0) * 2.0;
        let c = p0;
        let k3 = (a * a) * 2.0;
        let k2 = (a * b) * 3.0;
        for x in 0..512 {
            for y in 0..512 {
                let q = Vec2 {
                    x: x as f32,
                    y: y as f32,
                };
                let l = c - q;
                let k1 = (b * b) + (a * (l * 2.0));
                let k0 = b * l;
                let cubic = Polynomial {
                    coefficients: vec![k3, k2, k1, k0],
                };
                let mut candidate_intervals: Vec<Range> = vec![];
                let mut i = 0;
                let mut roots = vec![0.0, 1.0];
                while i < SAMPLE_POINTS + 1 {
                    let first = cubic.eval_horny(i as f32 / SAMPLE_POINTS as f32);
                    let second = cubic.eval_horny((i + 1) as f32 / SAMPLE_POINTS as f32);
                    if (first.abs() < EPS)
                        || (second.abs() < EPS)
                        || (first.signum() != second.signum())
                    {
                        candidate_intervals.push(Range {
                            lower: i as f32 / SAMPLE_POINTS as f32,
                            higher: (i as f32 + 1.0) / SAMPLE_POINTS as f32,
                        })
                    }
                    i += 1;
                }
                for i in candidate_intervals {
                    if let Some(root) = bisection(&cubic, i) {
                        roots.push(root);
                    }
                }
                let min_dist = get_min_dist(p0, p1, p2, q, &roots);
                plot(img, x, y, (min_dist * min_dist.signum()) / 400.0);
            }
        }
    }
    img.save("output2.png").unwrap();
}
const EPSILON: f32 = 0.0001;
fn get_min_dist(p0: Vec2, p1: Vec2, p2: Vec2, q: Vec2, roots: &[f32]) -> f32 {
    let mut min_dist = f32::MAX;

    for &t in roots {
        if t < 0.0 || t > 1.0 {
            continue;
        }

        let b = bezier(p0, p1, p2, t);
        let d = (b - q).magnitude();

        min_dist = min_dist.min(d);
    }

    min_dist
}

fn bezier(p0: Vec2, p1: Vec2, p2: Vec2, t: f32) -> Vec2 {
    let u = 1.0 - t;
    p0 * (u * u) + p1 * (2.0 * u * t) + p2 * (t * t)
}

fn bisection(f: &Polynomial, initial_guess: Range) -> Option<f32> {
    let mut a = initial_guess.lower;
    let mut b = initial_guess.higher;
    if f.eval_horny(a) * f.eval_horny(b) >= 0.0 {
        return None;
    }
    let mut c = a;
    while ((b - a) >= EPSILON) {
        c = (a + b) / 2.0;
        let c_value = f.eval_horny(c);
        if c_value == 0.0 {
            break;
        } else if c_value * f.eval_horny(a) < 0.0 {
            b = c;
        } else {
            a = c;
        }
    }
    Some(c)
}
fn newton(f: Polynomial) {}

//most likely did my vector stuff wrong
#[derive(Debug, Clone, Copy)]
struct Range {
    lower: f32,
    higher: f32,
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
    fn dot(&self, rhs: Self) -> f32 {
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
struct VectorValuedPolynomial<T: Vector> {
    coefficients: Vec<T>,
}
#[derive(Debug)]
struct Polynomial {
    coefficients: Vec<f32>,
}
impl Polynomial {
    fn eval(&self, x: f32) -> f32 {
        let mut total_value = 0.0;
        let mut current_degree = self.coefficients.len() - 1;
        for coefficient in &self.coefficients {
            let term = x.powi(current_degree as i32);
            total_value += term * coefficient;
            current_degree -= 1;
        }
        total_value
    }
    fn eval_horny(&self, x: f32) -> f32 {
        self.coefficients.iter().fold(0.0, |acc, &c| acc * x + c)
    }
    fn derirative(&self) -> Self {
        if self.coefficients.len() == 1 {
            return Self {
                coefficients: vec![],
            };
        }
        let mut current_degree = self.coefficients.len() - 1;
        let mut new_coefficients = vec![];
        for i in &self.coefficients {
            if current_degree == 0 {
                break;
            }
            new_coefficients.push(current_degree as f32 * i);
            current_degree -= 1;
        }
        Self {
            coefficients: new_coefficients,
        }
    }
}
impl Mul for Polynomial {
    fn mul(self, rhs: Self) -> Self::Output {
        let mut result = vec![0.0; self.coefficients.len() + rhs.coefficients.len() - 1];
        for (i, &a) in self.coefficients.iter().enumerate() {
            for (j, &b) in rhs.coefficients.iter().enumerate() {
                result[i + j] += a * b;
            }
        }
        Polynomial {
            coefficients: result,
        }
    }
    type Output = Self;
}

fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.min(min).max(max)
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
//t
