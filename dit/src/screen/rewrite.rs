use std::collections::{HashMap, HashSet, VecDeque};

use atlas_gen::{allocator::ShelfAllocator, atlas::Atlas};
use font_parser::{CellMetrics, TtfFont};
use image::Rgb;
use rand::seq::index;
use winit::dpi::LogicalSize;

use crate::{
    ansii::details::Attributes,
    renderer::{Mesh, buffer::Range, shader::Vertex},
};
const PLACEHOLDER: char = 'a';
///Limit for the amount of rows present. Since this is just for the row it means that the
///history_buffer will change the amount of memory based off of the cell amount in a row  
const ROW_LIMIT: usize = 1000;
///Never re allocates memory unless the user requests to do so. Will always have a capacity > 0
///Only self.capacity - 1 elements can be stored within the ring_buffer.
///
#[derive(Debug, Clone, Default)]
pub struct Cursor {
    pub x: usize,
    pub y: usize,
    pub visible: bool,
    pub blinking: bool,
}
#[derive(Clone)]
pub struct Cell {
    pub ch: char,
    pub attributes: Attributes,
}
impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: '\0',
            attributes: Attributes::default(),
        }
    }
}
#[inline(always)]
fn calculate_dims(window_size: LogicalSize<f32>, cell_metrics: &CellMetrics) -> (usize, usize) {
    let y_size = (window_size.height / cell_metrics.height).floor() as usize;
    let x_size = (window_size.width / cell_metrics.width).floor() as usize;
    return (y_size, x_size);
}
//Configs should have default opts,
trait Config: Default {}
pub struct ScreenConfig {
    disk_archival: bool,
    font_size: f32,
}
impl Config for ScreenConfig {}
impl Default for ScreenConfig {
    fn default() -> Self {
        Self {
            disk_archival: false,
            font_size: 10.0,
        }
    }
}
pub struct ArchiveBuffer {
    batch_size: usize,
    batch: Vec<u8>,
    file_table: HashMap<String, String>,
}

pub struct Screen {
    //All cells that have ever been generated
    pub history_buffer: VecDeque<Cell>,
    pub start_ptr: usize,
    //Cursor that defines the current location in the view_buffer
    pub cursor: Cursor,
    //In a given row the max amount of cells, limit of x essentially
    pub x_size: usize,
    //In a given column the max amount of cells, limit of y essentially
    pub y_size: usize,
    font: TtfFont,
    config: ScreenConfig,
    archive_buffer: Option<ArchiveBuffer>,
    cell_metric: CellMetrics,
    dirty_cells: HashSet<usize>,
    atlas: Atlas<char, Rgb<u8>, ShelfAllocator>,
    window_size: LogicalSize<f32>,
    mesh: Mesh,
}
trait Font {
}
struct FontManager<T: Font> {
    font_table: HashMap<&'static str, Font>,

}
///Holds a hashtable of fonts and is responsible for generating the font texture atlases. 
///
impl FontManager {

}


impl Screen {
    //for a given relative position return the absolute index
    pub fn new(font_size: f32, window_size: LogicalSize<f32>, ) {
        let cell_metrics = CellMetrics::new(font_size, font)
    }
    ///Resizing for the screen while keeping cell size the same, eg when the window has its dimensions reduced.
    pub fn window_resize(&mut self, window_size: LogicalSize<f32>) {
        let (new_x, new_y) = calculate_dims(window_size, &self.cell_metric);
        self.create_mesh();
    }
    ///Resizing for cells aka when the font size changes, eg when the user presses ctrl + or - to increase/decrease.
    pub fn cell_resize(&mut self, font_size: f32) {}

    #[inline(always)]
    pub fn get_absolute_index(&self) -> usize {
        let relative_index = self.cursor.y * self.x_size + self.cursor.x;
        self.start_ptr + relative_index
    }
    #[inline(always)]
    pub fn calc_absolute_index(&self, relative_x: usize, relative_y: usize) -> usize {
        let relative_index = relative_y * self.x_size + relative_x;
        self.start_ptr + relative_index
    }
    //goes down a row by iterating the start_ptr
    pub fn replace_oldest(&mut self) {}
    pub fn write_char(&mut self, ch: char) {
        match ch {
            '\x08' => {
                self.back_cursor(1);
                let index = self.get_absolute_index();
                self.history_buffer[index].ch = '\0';
                self.dirty_cells.insert(index);
            }
            c if !c.is_control() => {
                let index = self.get_absolute_index();
                self.history_buffer[index].ch = ch;
                self.advance_cursor(1);
                self.dirty_cells.insert(index);
                //writes at the current cursor position checking for validity
            }
            _ => {
                println!("Character does not have handler yet: {}", ch);
            }
        }
    }
    pub fn advance_cursor(&mut self, n: usize) {
        for _ in 0..n {
            self.cursor.x += 1;
            if self.cursor.x >= self.x_size {
                self.cursor.x = 0;
                //advance the y dim of the cursor
                if self.cursor.y + 1 < self.y_size {
                    //check if we are at row limit
                    //if yes we need to remove the oldest row in the history buffer
                    //what happens to said row is up to the user config (store to file or smth)
                    //next the
                    self.cursor.y = self.y_size;
                } else {
                    self.cursor.y += 1;
                }
            }
        }
    }
    pub fn back_cursor(&mut self, n: usize) {
        for _ in 0..n {
            if self.cursor.x == 0 {
                if self.cursor.y == 0 {
                    return;
                }
                self.cursor.y -= 1;
                self.cursor.x = self.x_size - 1;
            } else {
                self.cursor.x -= 1;
            }
        }
    }
}

impl Screen {
    fn create_mesh(&mut self) {
        self.mesh.reset();
        let vertices = &mut self.mesh.vertices;
        let indices = &mut self.mesh.indices;
        //Offset for writing index to the index buffer of mesh generation.
        let mut index_offset = 0u32;
        //Starts at viewable buffer index 0;
        let mut buffer_read_index = self.start_ptr;
        let mut cell_start_y = 0.0;
        for _ in 0..self.y_size {
            let mut cell_start_x = 0.0;
            let baseline_y = self.window_size.height - (cell_start_y + self.cell_metric.baseline);
            for _ in 0..self.x_size {
                let cell = &self.history_buffer[buffer_read_index];
                let (x0, y0, x1, y1);
                let (u0, v0, u1, v1);
                if !cell.ch.is_whitespace()
                    && let Some(gid) = self.font.lookup(cell.ch as u32)
                    && let Ok(Some(glyph)) = self.font.parse_gid(gid as u16)
                {
                    let header = glyph.get_header();
                    x0 = cell_start_x + header.x_min as f32 * self.cell_metric.scale;
                    y0 = baseline_y + header.y_max as f32 * self.cell_metric.scale;
                    x1 = cell_start_x + header.x_max as f32 * self.cell_metric.scale;
                    y1 = baseline_y + header.y_min as f32 * self.cell_metric.scale;
                    ([u0, v0], [u1, v1]) = self.atlas.get_uv(cell.ch);
                } else {
                    // whitespace OR lookup failure OR parse failure
                    x0 = cell_start_x;
                    y0 = cell_start_y;
                    x1 = cell_start_x + self.cell_metric.width;
                    y1 = cell_start_y + self.cell_metric.height;
                    u0 = 0.0;
                    v0 = 0.0;
                    u1 = 0.0;
                    v1 = 0.0;
                }
                (*vertices).extend_from_slice(&[
                    Vertex {
                        pos: [x0, y0],
                        uv: [u0, v0],
                    },
                    Vertex {
                        pos: [x0, y1],
                        uv: [u0, v1],
                    },
                    Vertex {
                        pos: [x1, y1],
                        uv: [u1, v1],
                    },
                    Vertex {
                        pos: [x1, y0],
                        uv: [u1, v0],
                    },
                ]);
                (*indices).extend_from_slice(&[
                    index_offset,
                    index_offset + 1,
                    index_offset + 2,
                    index_offset + 2,
                    index_offset + 3,
                    index_offset,
                ]);
                index_offset += 4;
                cell_start_x += self.cell_metric.width;
                buffer_read_index += 1;
            }
            cell_start_y += self.cell_metric.height;
        }
    }
    fn update_mesh(&mut self) -> Option<Vec<Range>> {
        if self.dirty_cells.is_empty() {
            return None;
        }
        //Might be over allocating but its not that big of an issue.
        let mut ranges = Vec::with_capacity(self.dirty_cells.len());
        for index in &self.dirty_cells {
            if let Some(cell) = self.history_buffer.get(*index) {
                let gid = self.font.lookup(cell.ch as u32).unwrap();
                let glyf = self
                    .font
                    .parse_gid(gid as u16)
                    .unwrap()
                    .clone()
                    .unwrap()
                    .get_header();
                let x = index % self.x_size;
                let y = index / self.x_size;
                let cell_start_x = x as f32 * self.cell_metric.width;
                let cell_start_y = y as f32 * self.cell_metric.height;
                let baseline_y =
                    self.window_size.height - (cell_start_y + self.cell_metric.baseline);
                let x0 = cell_start_x + glyf.x_min as f32 * self.cell_metric.scale;
                let y0 = baseline_y + glyf.y_max as f32 * self.cell_metric.scale;
                let x1 = cell_start_x + glyf.x_max as f32 * self.cell_metric.scale;
                let y1 = baseline_y + glyf.y_min as f32 * self.cell_metric.scale;
                let ([u0, v0], [u1, v1]) = if cell.ch == ' ' {
                    ([0.0, 0.0], [0.0, 0.0])
                } else {
                    self.atlas.get_uv(cell.ch)
                };
                let init_index = index * 4;
                let vx0 = Vertex {
                    pos: [x0, y0],
                    uv: [u0, v0],
                };
                let vx1 = Vertex {
                    pos: [x0, y1],
                    uv: [u0, v1],
                };
                let vx2 = Vertex {
                    pos: [x1, y1],
                    uv: [u1, v1],
                };
                let vx3 = Vertex {
                    pos: [x1, y0],
                    uv: [u1, v0],
                };
                self.mesh.vertices[init_index] = vx0;
                self.mesh.vertices[init_index + 1] = vx1;
                self.mesh.vertices[init_index + 2] = vx2;
                self.mesh.vertices[init_index + 3] = vx3;
                ranges.push(Range {
                    start: init_index,
                    end: init_index + 4,
                });
            }
        }
        self.dirty_cells.clear();
        Some(ranges)
    }
}
trait Renderable {
    ///Method to get the mesh from the
    fn get_mesh(&mut self);
    fn regenerate_mesh(&mut self);
}
