use std::{
    collections::{HashMap, HashSet},
    mem::offset_of,
};

use ash::vk;
use winit::{
    event::{ElementState, KeyEvent},
    keyboard::KeyCode,
};
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    pos: [f32; 2],
}
impl Vertex {
    fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(size_of::<Vertex>() as _)
            .input_rate(vk::VertexInputRate::VERTEX)
    }
    fn attribute_description() -> [vk::VertexInputAttributeDescription; 1] {
        let position_desc = vk::VertexInputAttributeDescription::default()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(offset_of!(Vertex, pos) as _);
        [position_desc]
    }
}

struct CellMesh {}
struct CellCache {
    cache: HashMap<char, CellMesh>,
}
impl CellCache {
    fn new() {}
}
#[derive(Default)]
pub struct Screen {
    pub rows: Vec<Row>,
    pub cursor: (usize, usize),
    pub max_rows: usize,
    pub max_cells: usize,
    pub damaged: Vec<usize>,
}
impl Screen {
    pub fn new(max_rows: usize, max_cells: usize) -> Self {
        let mut rows = Vec::new();
        for _ in 0..max_rows {
            let mut cells = Vec::new();
            for _ in 0..max_cells {
                cells.push(Cell {
                    glyph: Some('a'),
                    fg: [0, 0, 0, 0],
                    bg: [0, 0, 0, 0],
                    flags: CellFlags::empty(),
                });
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
        if pressed_keys.contains(&KeyCode::Backspace) {
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
                    let mut cell = *self.get_cell_mut();
                    cell.set_glyph(chars[0]);
                    self.damaged.push(self.cursor.1);
                    self.move_ahead();
                }
                //most likely an escape character will implement later
            }
        }
    }
    fn move_ahead(&mut self) {
        if self.cursor.0 + 1 < self.max_cells {
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
    fn get_cell_mut(&mut self) -> &mut Cell {
        &mut self.rows[self.cursor.1].cells[self.cursor.0]
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
pub struct Row {
    pub cells: Vec<Cell>,
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

//What the screen needs to accomplish
//Handles a cursor that moves across the screen
//Row cache to cache previous row text for faster reassembling of vertieces
//Updating the vertex buffer
//Cursor tracking along with highlighting text.

enum CursorState {
    Blinking,
    Highlight,
    Solid,
    Blank,
}
struct Cursor {
    x_limit: u32,
    y_limit: u32,
    x: u32,
    y: u32,
    cursor_state: CursorState,
}
impl Cursor {
    fn new(x_limit: u32, y_limit: u32) -> Self {
        Self {
            x_limit,
            y_limit,
            x: 0,
            y: 0,
            cursor_state: CursorState::Solid,
        }
    }
    //bounds updating
    fn update_bounds(&mut self, x_limit: u32, y_limit: u32) {
        self.x_limit = x_limit;
        self.y_limit = y_limit;
    }

    //these are for text insertion.
    //might be pointless to have
    fn move_ahead(&mut self, spaces: u32) {
        if self.x + spaces > self.x_limit {
            //calculate the rows to move down;
            let spaces_left = (self.x + spaces) - self.x_limit;
            let rows_to_move = spaces_left % self.x_limit;
            let cell_offset = spaces_left - (rows_to_move * self.x_limit);
            self.move_down(rows_to_move);
            self.x = cell_offset;
        } else {
            self.x += spaces;
        }
    }
    fn move_back(&mut self, spaces: u32) {
        //this means that the new cursor will be less than 0 or smth
        if self.x < spaces {
            let spaces_left = spaces - self.x;
            let rows_to_move = spaces_left & self.x_limit;
            let cell_offset = spaces_left - (rows_to_move * self.x_limit);
            self.move_up(rows_to_move);
            self.x = self.x_limit - cell_offset;
        } else {
            self.x -= spaces;
        }
    }
    //When true is returned it means the screen has to move up with the specified distance;
    fn move_up(&mut self, spaces: u32) -> (bool, u32) {
        if self.y < spaces {
            //means itll go back to 0
            return (true, spaces - self.y);
        } else {
            self.y -= spaces;
        }
        (false, 0)
    }
    fn move_down(&mut self, spaces: u32) -> (bool, u32) {
        //sends back a signal indicating if they have to extend the screen down
        if self.y + spaces > self.y_limit {
            let rows_to_move = (self.y + spaces) - self.y_limit;
            return (true, rows_to_move);
        } else {
            self.y += spaces;
        }
        (false, 0)
    }

    //this is for when the user uses a mouse to move the cursor to somewhere else
    //indicates if the op succeeded via a boolean
    fn change_position(&mut self, new_x: u32, new_y: u32) -> bool {
        if new_x > self.x_limit || new_y > self.y_limit {
            return false;
        } else {
            self.x = new_x;
            self.y = new_y;
            return true;
        }
    }

    fn change_state(&mut self, new_state: CursorState) {
        self.cursor_state = new_state;
    }
}
enum Mode {
    Insert,
    Cursor,
}
struct Cursor2 {}
impl Cursor2 {}

struct Screen2 {
    cursor: Cursor2,
    mode: Mode,

}

impl Screen2 {}
