use std::ops::Mul;

use font_parser::Vec2;

#[derive(Debug)]
pub struct VectorPolynomial {
    x: Polynomial,
    y: Polynomial,
}
impl VectorPolynomial {
    pub fn eval(&self, t: f32) -> Vec2 {
        Vec2 {
            x: self.x.eval_horner(t),
            y: self.y.eval_horner(t),
        }
    }
    pub fn derivative(&self) -> Self {
        Self {
            x: self.x.derivative(),
            y: self.y.derivative(),
        }
    }
}
#[derive(Debug)]
pub struct Polynomial {
    coefficients: Vec<f32>,
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
const EPSILON: f32 = 0.0001;
#[derive(Debug, Clone, Copy)]
struct Range {
    lower: f32,
    higher: f32,
}
fn bisection(f: &Polynomial, initial_guess: Range) -> Option<f32> {
    let mut a = initial_guess.lower;
    let mut b = initial_guess.higher;
    if f.eval_horner(a) * f.eval_horner(b) >= 0.0 {
        return None;
    }
    let mut c = a;
    while ((b - a) >= EPSILON) {
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
