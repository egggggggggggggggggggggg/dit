use std::{cell, collections::HashMap, env::current_exe, fmt::Error};

use font_parser::{Hhea, TtfFont, hhea};

enum GenericError {
    Placeholder,
}
#[derive(Default)]
struct Point {
    x: u32,
    y: u32,
}
//Things required
//Cursor position maintaining
//Buffer for text holding
//This is handled by screen where it has the specific cells with the glyphs
//Cursor can be position at 0  and limit
//When it reaches the limit put the cursor to the next row
//

struct Screen {
    //maintains a buffer of the previous rows that have been hidden
    previous_rows: Vec<RectLine>,
    //maintains a buffer of the rows that are ahead
    //but are hidden because its been scrolled up
    ahead_rows: Vec<RectLine>,
    rect_lines: Vec<RectLine>,
    cursor: Point,
    row_count: u32,
    cell_count: u32,
}
impl Screen {
    fn new(cell_count: u32, row_count: u32, hhea: &Hhea) -> Self {
        let mut rect_lines = Vec::with_capacity(row_count as usize);
        for _ in 0..row_count {
            let row = RectLine::with_cells(cell_count);
            rect_lines.push(row);
        }
        Self {
            rect_lines,
            cursor: Point::default(),
            row_count,
            cell_count,
            ahead_rows: Vec::new(),
            previous_rows: Vec::new(),
        }
    }
    fn move_cursor(&mut self, x_delta: i32, y_delta: i32) {
        let mut cursor = &self.cursor;
        let new_x_pos = cursor.x as i32 + x_delta;
        let new_y_pos = cursor.y as i32 + y_delta;
        //return a signal indicating that the screen should scroll back to the previous row if its in the cached buffer
        if new_x_pos > self.cell_count as i32 {
            //go down
        } else if new_x_pos < self.cell_count as i32 {
            //go up
        }
        if new_y_pos > self.row_count as i32 {
            //go up to the next hidden row
        } else if new_y_pos < self.row_count as i32 {
        }
    }
    fn hide_row(delta: i32) {
        //sends a delta of the rows to hide
        //
    }
    fn get_mut_rect(&mut self, row_num: u32, cell_num: u32) -> Result<&mut Rect, GenericError> {
        let rect_line = self
            .rect_lines
            .get_mut(row_num as usize)
            .ok_or(GenericError::Placeholder)?;
        rect_line
            .get_mut_rect(cell_num)
            .ok_or(GenericError::Placeholder)
    }
    fn set_char(&mut self) {}
    fn resize(new_x: u32, new_y: u32) {
        //cerates a hlding buffer for the new data to be moved into temporarily
        //makes a new buffer where the old wll be placed
    }
    //maintain a cursor that tells the
}

#[derive(Default)]
struct RectLine {
    rects: Vec<Rect>,
}
impl RectLine {
    fn new() -> Self {
        Self { rects: Vec::new() }
    }
    fn with_cells(cell_count: u32) -> Self {
        let mut rects = Vec::with_capacity(cell_count as usize);
        for _ in 0..cell_count {
            rects.push(Rect::default());
        }
        Self { rects }
    }
    fn get_mut_rect(&mut self, cell_num: u32) -> Option<&mut Rect> {
        self.rects.get_mut(cell_num as usize)
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
//maintiain a cursor
