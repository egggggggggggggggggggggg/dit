#[cfg(test)]
mod tests {
    use math::calc::{solve_cubic, solve_quadratic};

    use super::*;

    const EPS: f64 = 1e-9;

    fn eval_cubic(a: f64, b: f64, c: f64, d: f64, x: f64) -> f64 {
        ((a * x + b) * x + c) * x + d
    }

    fn eval_quadratic(a: f64, b: f64, c: f64, x: f64) -> f64 {
        (a * x + b) * x + c
    }

    fn approx_eq(x: f64, y: f64) -> bool {
        (x - y).abs() < EPS
    }

    #[test]
    fn test_quadratic_solver() {
        // ax^2 + bx + c = 0
        let tests = vec![
            (1.0, 0.0, -1.0), // x = ±1
            (1.0, -2.0, 1.0), // x = 1 (double)
            (0.0, 2.0, -4.0), // linear: x = 2
            (0.0, 0.0, 0.0),  // infinite solutions
            (1.0, 0.0, 1.0),  // no real roots
        ];

        for (a, b, c) in tests {
            let (sol, infinite) = solve_quadratic(a, b, c);

            if infinite {
                // if infinite solutions, we can't check roots
                continue;
            }

            for x in sol {
                assert!(approx_eq(eval_quadratic(a, b, c, x), 0.0));
            }
        }
    }

    #[test]
    fn test_cubic_solver() {
        // ax^3 + bx^2 + cx + d = 0
        let tests = vec![
            (1.0, 0.0, 0.0, 0.0),    // x = 0 (triple)
            (1.0, -6.0, 11.0, -6.0), // x = 1,2,3
            (1.0, 0.0, -1.0, 0.0),   // x = -1,0,1
            (0.0, 1.0, -5.0, 6.0),   // quadratic: x = 2,3
            (0.0, 0.0, 2.0, -8.0),   // linear: x = 4
        ];

        for (a, b, c, d) in tests {
            let (sol, _inf) = solve_cubic(a, b, c, d);

            for x in sol {
                assert!(approx_eq(eval_cubic(a, b, c, d, x), 0.0));
            }
        }
    }

    #[test]
    fn random_cubic_test() {
        use rand::Rng;

        let mut rng = rand::thread_rng();

        for count in 0..1000 {
            let a: f64 = rng.gen_range(-10.0..10.0);
            let b: f64 = rng.gen_range(-10.0..10.0);
            let c: f64 = rng.gen_range(-10.0..10.0);
            let d: f64 = rng.gen_range(-10.0..10.0);

            let (roots, _) = solve_cubic(a, b, c, d);

            for x in roots {
                let v = eval_cubic(a, b, c, d, x);
                assert!(
                    approx_eq(v, 0.0),
                    "Failed for a={}, b={}, c={}, d={}, root={} at tes={}",
                    a,
                    b,
                    c,
                    d,
                    x,
                    count,
                );
            }
        }
    }
}
