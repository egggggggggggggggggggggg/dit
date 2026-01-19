use crate::lalg::Vec2;
use std::ops::Mul;
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
    pub coefficients: Vec<f32>,
}
impl Polynomial {
    pub fn eval(&self, x: f32) -> f32 {
        let mut total_value = 0.0;
        let mut current_degree = self.coefficients.len() - 1;
        for coefficient in &self.coefficients {
            let term = x.powi(current_degree as i32);
            total_value += term * coefficient;
            current_degree -= 1;
        }
        total_value
    }
    pub fn eval_horner(&self, x: f32) -> f32 {
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
            new_coefficients.push(current_degree as f32 * i);
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
    pub fn find_roots(&self, sample_amount: u32, epsilon: f32) -> Vec<f32> {
        //this method works on any degree but im using it for cubic solving
        let mut candidate_intervals: Vec<Range> = vec![];
        let mut i = 0;
        let mut roots = vec![0.0, 1.0];
        while i < sample_amount + 1 {
            let first = self.eval_horner(i as f32 / sample_amount as f32);
            let second = self.eval_horner((i + 1) as f32 / sample_amount as f32);
            if (first.abs() < epsilon)
                || (second.abs() < epsilon)
                || (first.signum() != second.signum())
            {
                candidate_intervals.push(Range {
                    lower: i as f32 / sample_amount as f32,
                    higher: (i + 1) as f32 / sample_amount as f32,
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
    pub lower: f32,
    pub higher: f32,
}
#[inline(always)]
pub fn bisection(f: &Polynomial, initial_guess: Range, epsilon: f32) -> Option<f32> {
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
pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.min(min).max(max)
}
#[inline(always)]
pub fn median(a: u8, b: u8, c: u8) -> u8 {
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
