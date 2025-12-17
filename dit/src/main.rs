use image::{ImageBuffer, Rgb};
fn main() {
    println!("Hello, world!");
    let p0 = Position { x: 10.0, y: 5.0 };
    let p1 = Position { x: 100.0, y: 50.0 };
    let p2 = Position { x: 400.0, y: 12.0 };
    let qbc = QBezierCurve {
        points: [p0, p1, p2],
    };
    draw_curve(qbc, 1000);
}
struct Position {
    x: f32,
    y: f32,
}
struct QBezierCurve {
    points: [Position; 3],
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
        let whole_x = x.floor();
        let frac_x = x - whole_x;
        let whole_y = y.floor();
        let frac_y = y - whole_y;
        let pixel = img.get_pixel_mut(whole_x as u32, whole_y as u32);
        //perform a bounds check letter for anti-aliasing;
        *pixel = Rgb([255 as u8, 255 as u8, 128])
    }

    img.save("output.png").unwrap();
}
fn generate_msdf() {}
