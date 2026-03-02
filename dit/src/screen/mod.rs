use font_parser::TtfFont;

use crate::ansii::{Handler, details::Attributes, utf_decoder::Utf8Decoder};

const BLINK_DURATION: f64 = 0.5;
#[derive(Debug, Default, Clone)]
pub struct Cursor {
    pub row: usize,
    pub col: usize,
    pub visible: bool,
    pub blinking: bool,
}
#[derive(Default)]
struct Cell {
    ch: char,
    cell_attr: Attributes,
}
struct ScreenConfig {
    font_size: f64,
}
pub struct Screen<'a> {
    pub cells: Vec<Cell>,
    pub cursor: Cursor,
    pub row_size: usize,
    pub col_size: usize,
    pub accumulator: Utf8Decoder,
    font: &'a TtfFont,
    config: ScreenConfig,
}
// An arbitrary character for monospace fonts
const PLACEHOLDER: char = 'a';
impl<'a> Screen<'a> {
    fn new(row_size: usize, col_size: usize, font: &'a TtfFont, config: ScreenConfig) -> Self {
        Self {
            cells: Vec::new(),
            cursor: Cursor::default(),
            row_size,
            col_size,
            accumulator: Utf8Decoder::new(),
            font,
            config,
        }
    }
    fn construct_mesh(&mut self) {
        let units_per_em = self.font.head.units_per_em;
        let gid = self.font.lookup(PLACEHOLDER as u32).unwrap();
        let cell_advance = self.font.hmtx.metric_for_glyph(gid as u16).advance_width;
        let cell_ascent = self.font.hhea.ascent;
        let cell_descent = self.font.hhea.descent;
        let cell_height = cell_ascent - cell_descent + self.font.hhea.line_gap;
        let 
        for (index, cell) in self.cells.iter().enumerate() {
            let col_pos = index % self.row_size;
            let row_pos = (index - col_pos) / self.row_size;
        }
    }
    fn resize(&mut self, new_row: usize, new_col: usize) {
        self.col_size = new_col;
        self.row_size = new_row;
        self.cells.resize_with(new_col * new_row, Default::default);
    }
}

impl<'a> Handler for Screen<'a> {
    fn cursor_up(&mut self, n: u16) {
        self.cursor.row = self.cursor.row.saturating_sub(n as usize);
    }
    fn cursor_down(&mut self, n: u16) {
        let max_row = self.row_size - 1;
        self.cursor.row = (self.cursor.row + n as usize).min(max_row);
    }
    fn cursor_right(&mut self, n: u16) {
        let max_col = self.col_size - 1;
        self.cursor.col = self.cursor.col.saturating_add(n as usize).min(max_col);
    }

    fn cursor_left(&mut self, n: u16) {
        self.cursor.col = self.cursor.col.saturating_sub(n as usize);
    }
    fn accumluate_utf8(&mut self, byte: u8) {
        if let Some(ch) = self.accumulator.decode(byte) {
            //
            // writes the character to the visual buffer
        }
    }
    fn bell() {}
    fn char_attributes(&mut self, params: &smallvec::SmallVec<[u16; 8]>) {}
    fn csi() {}

    fn cursor_position(&mut self, new_x: u16, new_y: u16) {}
    fn device_status_report(&mut self, param: u16) {}
    fn execute(&mut self, ctl_seq: u8) {}
    fn handle_osc(&mut self, u: &Vec<u8>) {}
    fn next_line(&mut self) {}
    fn previous_line(&mut self) {}
}
