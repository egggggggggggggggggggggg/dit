use font_parser::{Hhea, hhea};

struct RectLine {
    rects: Vec<Rect>,
}
impl RectLine {
    fn calculate_vertices(hhea: &Hhea) {
        let ascent = hhea.ascent;
        let descent = hhea.descent;
        let line_gap = hhea.line_gap;
    }
}
struct Rect {
    ch: char,
    fg: Color,
    bg: Color,
    flags: RectFlags,
}
bitflags::bitflags!(
    #[derive(Default, Clone, Copy)]
    pub struct RectFlags: u8 {
        const UNDERCURL = 0x0000;
        const UNDERLINE = 0x0001;
        const HIGHLIGHT = 0x0002;
    }
);
struct Color {
    value: [u8; 3],
}
impl Color {
    const RED: Self = Self {
        value: [255, 255, 255],
    };
    const WHITE: Self = Self {
        value: [255, 255, 255],
    };
    const BLACK: Self = Self {
        value: [0u8, 0u8, 0u8],
    };
}
impl Default for Rect {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: Color::WHITE,
            bg: Color::BLACK,
            flags: RectFlags::empty(),
        }
    }
}
impl Rect {}
