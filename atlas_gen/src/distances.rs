#[derive(Clone)]
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
#[derive(Clone)]
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
