use std::env::current_exe;

use font_parser::{Hhea, TtfFont, hhea};

struct RectLine {
    rects: Vec<Rect>,
    cell_height: u16,
    row_num: u16,
}
impl RectLine {
    fn new(hhea: &Hhea, row_num: u16) -> Self {
        //cell height should never be a negative number, if it is it means smth went wrong
        let cell_height = (hhea.ascent + hhea.descent) as u16;
        //row num acts as a way to generate the vertices for the given line
        //this is required as vertices are relative to the scren;
        Self {
            rects: Vec::new(),
            cell_height,
            row_num,
        }
    }
    //if the font is monospaced line_gap can be set to be the same value for all characters
    //otherwise each character has to have their linegap derived from this:
    //linespace = ascent - descent + line_gap
    //TtfFont has to have the required characters in the text parsed.
    //If not force it to and then check again

    //
    fn calculate_vertices(&mut self, ttf_font: &TtfFont, target_px: f32, total_rows: u16) {
        let hhea = &ttf_font.hhea;
        let units_per_em = ttf_font.head.units_per_em;
        let scale = target_px as f32 / units_per_em as f32;

        //get the basic info
        let mut current_cell = 0;
        //range of 0 to 1
        let cell_height = self.row_num as f32 / total_rows as f32;
        let mut pen = ();

        //offset in screen coords
        let mut current_advance_width = 0;

        //new_va
        let baseline = self.row_num as i16 * self.cell_height as i16 + hhea.ascent;
        for rect in &self.rects {
            let gid = ttf_font.lookup(rect.ch as u32).unwrap();
            let hmetrics = ttf_font.hmtx.metric_for_glyph(gid as u16);
            let bbox = ttf_font.get_glyf_header(gid as u16).unwrap();
            current_advance_width += hmetrics.advance_width;
            println!("baseline: {}", baseline);
            //renders over the top of the cell

            current_cell += 1;
        }
    }
    //this should probably be outside as its possible the user can have padding
    //enabled for the screen meaning all the vertices would have to be offset again
    fn resize(&mut self, target_cell_size: u32) -> Vec<Rect> {
        //splices the given rectangles and return it to be plaecd into a new RectLine
        Vec::new()
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
