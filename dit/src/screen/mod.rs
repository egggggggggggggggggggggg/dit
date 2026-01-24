use std::collections::{HashMap, HashSet};

use winit::{
    event::{ElementState, KeyEvent},
    keyboard::KeyCode,
};

#[derive(Default)]
pub struct Screen {
    rows: Vec<Row>,
    cursor: (usize, usize),
    max_rows: usize,
    max_cells: usize,
    damaged: Vec<usize>,
}
impl Screen {
    pub fn new(max_rows: usize, max_cells: usize) -> Self {
        let mut rows = Vec::new();
        for _ in 0..max_rows {
            let mut cells = Vec::new();
            for _ in 0..max_cells {
                cells.push(Cell::default());
            }
            rows.push(Row { cells });
        }
        Self {
            rows,
            cursor: (0, 0),
            max_rows,
            max_cells,
            damaged: Vec::new(),
        }
    }
    pub fn change(&mut self, key_event: &KeyEvent, pressed_keys: &HashSet<KeyCode>) {
        if pressed_keys.contains(&KeyCode::Delete) {
            let row = self.rows.get(self.cursor.1).unwrap();
            let mut cell = *row.cells.get(self.cursor.0).unwrap();
            cell.delete_glyph();
            self.damaged.push(self.cursor.1);
            self.move_back();
        }
        if pressed_keys.contains(&KeyCode::ArrowUp) {
            if !(self.cursor.1 == 0 || self.cursor.1 == self.max_rows) {
                self.cursor.1 += 1;
            }
        }
        if pressed_keys.contains(&KeyCode::ArrowDown) {
            if !(self.cursor.1 == 0 || self.cursor.1 == self.max_rows) {
                self.cursor.1 -= 1;
            }
        }
        if pressed_keys.contains(&KeyCode::ArrowRight) {
            if !(self.cursor.0 == 0 || self.cursor.0 == self.max_cells) {
                self.cursor.0 += 1;
            }
        }
        if pressed_keys.contains(&KeyCode::ArrowLeft) {
            if !(self.cursor.0 == 0 || self.cursor.0 == self.max_cells) {
                self.cursor.0 -= 1;
            }
        }
        match &key_event.text {
            None => {}
            Some(str) => {
                let chars: Vec<char> = str.chars().collect();
                if chars.len() == 1 {
                    let mut cell = *self.get_cell();
                    cell.set_glyph(chars[0]);
                    self.damaged.push(self.cursor.1);
                    self.move_ahead();
                }
                //most likely an escape character will implement later
            }
        }
    }
    fn move_ahead(&mut self) {
        if self.cursor.0 < self.max_cells {
            self.cursor.0 += 1;
        } else if self.cursor.1 < self.max_rows {
            self.cursor.1 += 1;
        }
    }
    fn move_back(&mut self) {
        if self.cursor.0 == 0 && self.cursor.1 > 0 {
            self.cursor.1 -= 1;
        } else {
            self.cursor.0 -= 1;
        }
    }
    fn get_cell(&mut self) -> &Cell {
        let row = self.rows.get(self.cursor.1).unwrap();
        row.cells.get(self.cursor.0).unwrap()
    }
}
#[derive(Default, Clone, Copy)]
pub struct Cell {
    pub glyph: Option<char>,
    pub fg: Color,
    pub bg: Color,
    pub flags: CellFlags,
}
impl Cell {
    pub fn delete_glyph(&mut self) {
        self.glyph = None;
    }
    pub fn set_glyph(&mut self, char: char) {
        self.glyph = Some(char);
    }
}
#[derive(Default, Clone)]
struct Row {
    cells: Vec<Cell>,
}
bitflags::bitflags!(
    #[derive(Default, Clone, Copy)]
    pub struct CellFlags: u8 {
        const ESCAPE = 0x0000;
        const TAB = 0x0001;
        const NEWLINE = 0x0002;
    }
);
type Color = [u8; 4];
