use font_parser::ttf_parse::TtfFont;
fn main() {
    let res = median(122, 3333, 0);
    println!("res: {}", res);
}

fn median<T>(a: T, b: T, c: T) -> T
where
    T: Ord + Copy,
{
    (a.min(b)).max(b.min(c))
}
