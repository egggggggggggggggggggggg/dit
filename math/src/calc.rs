use crate::lalg::Vec2;
use core::panic;
use std::{f64, ops::Mul};
#[derive(Debug, Copy, Clone)]
pub enum Degree {
    Constant,
    Linear,
    Quadratic,
    Cubic,
    Quartic,
    Quintic,
    Invalid,
}
impl From<u32> for Degree {
    fn from(value: u32) -> Self {
        match value {
            1 => Self::Constant,
            2 => Self::Linear,
            3 => Self::Quadratic,
            4 => Self::Cubic,
            5 => Self::Quintic,
            6 => Self::Quartic,
            _ => Self::Invalid,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Polynomial {
    pub coefficients: Vec<f64>,
}
impl Polynomial {
    pub fn eval(&self, x: f64) -> f64 {
        let mut total_value = 0.0;
        let mut current_degree = self.coefficients.len() - 1;
        for coefficient in &self.coefficients {
            let term = x.powi(current_degree as i32);
            total_value += term * coefficient;
            current_degree -= 1;
        }
        total_value
    }
    pub fn eval_horner(&self, x: f64) -> f64 {
        self.coefficients.iter().fold(0.0, |acc, &c| acc * x + c)
    }
    pub fn derivative(&self) -> Self {
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
            new_coefficients.push(current_degree as f64 * i);
            current_degree -= 1;
        }
        Self {
            coefficients: new_coefficients,
        }
    }
    pub fn degree(&self) -> Degree {
        Degree::from(self.coefficients.len() as u32)
    }
    //maybe make this more flexible
    //options might be
    //sampling rate
    //range
    pub fn find_roots(&self, sample_amount: u32, epsilon: f64) -> Vec<f64> {
        //this method works on any degree but im using it for cubic solving
        let mut candidate_intervals: Vec<Range> = vec![];
        let mut i = 0;
        let mut roots = vec![0.0, 1.0];
        while i < sample_amount + 1 {
            let first = self.eval_horner(i as f64 / sample_amount as f64);
            let second = self.eval_horner((i + 1) as f64 / sample_amount as f64);
            if (first.abs() < epsilon)
                || (second.abs() < epsilon)
                || (first.signum() != second.signum())
            {
                candidate_intervals.push(Range {
                    lower: i as f64 / sample_amount as f64,
                    higher: (i + 1) as f64 / sample_amount as f64,
                })
            }
            i += 1;
        }
        for i in candidate_intervals {
            if let Some(root) = bisection(&self, i, 0.001) {
                roots.push(root);
            }
        }
        roots
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
#[derive(Debug, Clone, Copy)]
pub struct Range {
    pub lower: f64,
    pub higher: f64,
}
#[inline(always)]
pub fn bisection(f: &Polynomial, initial_guess: Range, epsilon: f64) -> Option<f64> {
    let mut a = initial_guess.lower;
    let mut b = initial_guess.higher;
    if f.eval_horner(a) * f.eval_horner(b) >= 0.0 {
        return None;
    }
    let mut c = a;
    while (b - a) >= epsilon {
        c = (a + b) / 2.0;
        let c_value = f.eval_horner(c);
        if c_value == 0.0 {
            break;
        } else if c_value * f.eval_horner(a) < 0.0 {
            b = c;
        } else {
            a = c;
        }
    }
    Some(c)
}
#[inline(always)]
pub fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.min(min).max(max)
}
#[inline(always)]
pub fn median<T: PartialOrd + Copy>(a: T, b: T, c: T) -> T {
    if a < b {
        if b < c {
            b
        } else if a < c {
            c
        } else {
            a
        }
    } else {
        if a < c {
            a
        } else if b < c {
            c
        } else {
            b
        }
    }
}

pub fn solve_cubic_normed(a: f64, b: f64, c: f64) -> Vec<f64> {
    let mut a = a.clone();
    //No with_capacity() as the amount of solution varies and is important info
    let mut solutions = Vec::new();
    let a2 = a * a;
    let mut q = (a2 - 3.0 * b) / 9.0;
    let r = (2.0 * a * a2 - 9.0 * a * b + 27.0 * c) / 54.0;
    let r2 = r * r;
    let q3 = q * q * q;
    a *= 1.0 / 3.0;
    if r2 < q3 {
        let t = {
            let mut t = r / q3.sqrt();
            if t < -1.0 {
                t = -1.0
            }
            if t > 1.0 {
                t = 1.0
            }
            t.acos()
        };
        q = -2.0 * q.sqrt();
        solutions.push(q * (1.0 / 3.0 * t).cos() - a);
        solutions.push(q * (1.0 / 3.0 * (t + 2.0 * f64::consts::PI)).cos() - a);
        solutions.push(q * (1.0 / 3.0 * (t - 2.0 * f64::consts::PI)).cos() - a);
    } else {
        let u = {
            let z = if r < 0.0 { 1.0 } else { -1.0 };
            z * (r.abs() + (r2 - q3).sqrt()).powf(1.0 / 3.0)
        };
        let v = if u == 0.0 { 0.0 } else { q / u };
        solutions.push((u + v) - a);
        if u == v || (u - v).abs() < 1e-12 * (u + v).abs() {
            solutions.push(-(0.5 * (u + v) - a));
        }
    }
    solutions
}
pub fn solve_cubic(a: f64, b: f64, c: f64, d: f64) -> (Vec<f64>, bool) {
    if a != 0.0 {
        let bn = b / a;
        if bn.abs() < 1e6 {
            return (solve_cubic_normed(bn, c / a, d / a), false);
        }
    }
    solve_quadratic(b, c, d)
}

pub fn solve_quadratic(a: f64, b: f64, c: f64) -> (Vec<f64>, bool) {
    //No with_capacity() as the amount of solution varies and is important info
    let mut solutions = Vec::new();
    //linear
    if a == 0.0 || b.abs() > 1e12 * a.abs() {
        if b == 0.0 {
            if c == 0.0 {
                //Infinite solutions
                return (solutions, true);
            }
            //0 solution
            return (solutions, false);
        }
        //1 solution
        solutions.push(-c / b);
        return (solutions, false);
    }
    let mut discriminant = b * b - 4.0 * a * c;
    if discriminant > 0.0 {
        discriminant = discriminant.sqrt();
        solutions.push((-b + discriminant) / (2.0 * a));
        solutions.push((-b - discriminant) / (2.0 * a));
    } else if discriminant == 0.0 {
        solutions.push(-b / (2.0 * a));
    }
    return (solutions, false);
}
