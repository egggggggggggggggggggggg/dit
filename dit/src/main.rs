use image::{ImageBuffer, Rgb};
fn main() {
    println!("Hello, world!");
    let p0 = Position { x: 0.0, y: 0.0 };
    let p1 = Position { x: 256.0, y: 256.0 };
    let p2 = Position { x: 512.0, y: 512.0 };
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
