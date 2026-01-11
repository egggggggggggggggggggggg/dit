use font_parser::ttf_parse::TtfFont;
fn main() {
    let font = TtfFont::new("../JetBrainsMonoNerdFontMono-Regular.ttf").unwrap();
    let atlas = atlas_gen::entry();
}
struct Cell {
    glyph_id: u16,
    fg_color: [u8; 4],
    bg_color: [u8; 4],
    flags: u8,
}
