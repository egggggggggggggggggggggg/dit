use atlas_gen::entry;
use font_parser::ttf_parse::TtfFont;
fn main() {
    entry();
}

fn median<T>(a: T, b: T, c: T) -> T
where
    T: Ord + Copy,
{
    (a.min(b)).max(b.min(c))
}
