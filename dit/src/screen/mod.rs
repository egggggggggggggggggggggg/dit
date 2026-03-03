use std::{collections::HashSet, ops::Range};

use atlas_gen::{allocator::ShelfAllocator, atlas::Atlas};
use font_parser::TtfFont;
use image::Rgb;

use crate::{
    ansii::{Handler, details::Attributes, utf_decoder::Utf8Decoder},
    renderer::{Mesh, buffer::DynamicBuffer, shader::Vertex},
};

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
pub struct CellMetrics {
    pub width: f32,
    pub height: f32,
    pub baseline: f32,
    pub underline_pos: f32,
    pub underline_thickness: f32,
    pub font_size: f32,
    pub scale: f32,
}
impl CellMetrics {
    pub fn new(font_size: f32, font: &TtfFont) -> Self {
        let units_per_em = font.head.units_per_em;
        let gid = font.lookup(PLACEHOLDER as u32).unwrap();
        let arbitrary_metrics = font.hmtx.metric_for_glyph(gid as u16);
        let cell_advance = arbitrary_metrics.advance_width;
        let cell_ascent = font.hhea.ascent;
        let cell_descent = font.hhea.descent;
        let cell_height = cell_ascent - cell_descent + font.hhea.line_gap;
        let scale = font_size / units_per_em as f32;
        let cell_width_px = cell_advance as f32 * scale;
        let cell_height_px = cell_height as f32 * scale;
        let baseline_y = cell_ascent as f32 * scale;
        let underline_offset_px = font.post.underline_position as f32 * scale;
        let underline_thickness = font.post.underline_thickness as f32 * scale;
        Self {
            font_size,
            width: cell_width_px,
            height: cell_height_px,
            baseline: baseline_y,
            underline_pos: underline_offset_px,
            underline_thickness,
            scale,
        }
    }
}
pub struct Screen {
    pub cells: Vec<Cell>,
    pub cursor: Cursor,
    pub row_size: usize,
    pub col_size: usize,
    pub accumulator: Utf8Decoder,
    font: TtfFont,
    cell_metrics: CellMetrics,
    dirty_cells: HashSet<usize>,
    atlas: Atlas<char, Rgb<u8>, ShelfAllocator>,
    mesh: Mesh,
}
// An arbitrary character for monospace fonts
const PLACEHOLDER: char = 'a';
#[inline(always)]
fn calculate_dims(
    logical_screen_size: winit::dpi::LogicalSize<f32>,
    cell_metrics: &CellMetrics,
) -> (usize, usize) {
    let row_size = (logical_screen_size.height / cell_metrics.height).floor() as usize;
    let col_size = (logical_screen_size.width / cell_metrics.width).floor() as usize;
    return (row_size, col_size);
}
impl Screen {
    // for the cursor that could be implemented via a static quad and controlled
    //via a ubo that specifies properties sorta like how instancing works
    // eg positioin and then the specific inherent properties
    // issue is how to write the shader to allow for this
    fn new(
        font_size: f32,
        font: TtfFont,
        texture_atlas: Atlas<char, Rgb<u8>, ShelfAllocator>,
        logical_screen_size: winit::dpi::LogicalSize<f32>,
    ) -> Self {
        let cell_metrics = CellMetrics::new(font_size, &font);
        let (row_size, col_size) = calculate_dims(logical_screen_size, &cell_metrics);
        Self {
            cells: Vec::new(),
            cursor: Cursor::default(),
            row_size,
            col_size,
            accumulator: Utf8Decoder::new(),
            font,
            cell_metrics,
            dirty_cells: HashSet::new(),
            atlas: texture_atlas,
            mesh: Mesh::default(),
        }
    }
    fn resize(&mut self, new_font_size: f32, logical_screen_size: winit::dpi::LogicalSize<f32>) {
        if new_font_size != self.cell_metrics.font_size {
            // recalculate cell_metrics if font_size changes
            self.cell_metrics = CellMetrics::new(new_font_size, &self.font)
        }
        let (new_row_size, new_col_size) = calculate_dims(logical_screen_size, &self.cell_metrics);
        self.col_size = new_col_size;
        self.row_size = new_row_size;
        self.construct_mesh();
        // must reconstruct the mesh from scratch
    }
    fn construct_mesh(&mut self) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut index_offset = 0u32;
        let mut y_cell = 0.0;
        for row in 0..self.row_size {
            let mut x_cell = 0.0;
            let row_start = row * self.col_size;
            let row_slice = &self.cells[row_start..row_start + self.row_size];
            for cell in row_slice {
                if cell.ch == ' ' {
                    // empty
                } else {
                    let gid = self.font.lookup(cell.ch as u32).unwrap();
                    let glyf = self
                        .font
                        .parse_gid(gid as u16)
                        .unwrap()
                        .clone()
                        .unwrap()
                        .get_header();
                    let baseline_x = x_cell;
                    let baseline_y = y_cell + self.cell_metrics.baseline;
                    let x0 = baseline_x + glyf.x_min as f32 * self.cell_metrics.scale;
                    let y0 = baseline_y + glyf.y_max as f32 * self.cell_metrics.scale;
                    let x1 = baseline_x + glyf.x_max as f32 * self.cell_metrics.scale;
                    let y1 = baseline_y + glyf.y_min as f32 * self.cell_metrics.scale;
                    let ([u0, v0], [u1, v1]) = if cell.ch == ' ' {
                        ([0.0, 0.0], [0.0, 0.0])
                    } else {
                        self.atlas.get_uv(cell.ch)
                    };
                    vertices.push(Vertex {
                        pos: [x0, y0],
                        uv: [u0, v0],
                    });
                    vertices.push(Vertex {
                        pos: [x0, y1],
                        uv: [u0, v1],
                    });
                    vertices.push(Vertex {
                        pos: [x1, y1],
                        uv: [u1, v1],
                    });
                    vertices.push(Vertex {
                        pos: [x1, y0],
                        uv: [u1, v0],
                    });
                    indices.push(index_offset + 0);
                    indices.push(index_offset + 1);
                    indices.push(index_offset + 2);
                    indices.push(index_offset + 2);
                    indices.push(index_offset + 3);
                    indices.push(index_offset + 0);
                    index_offset += 4;
                    x_cell += self.cell_metrics.width;
                }
                y_cell += self.cell_metrics.height;
                // row, col directly available
            }
        }
        self.mesh.indices = indices;
        self.mesh.vertices = vertices;
    }
    fn update_mesh(&mut self, vertex_buffer: &mut DynamicBuffer) {
        if self.dirty_cells.is_empty() {
            return;
        }
        // Theoretically the index buffer shouldn't need updating unless
        for index in &self.dirty_cells {
            if let Some(cell) = self.cells.get(*index) {
                // identify the position inthe ver
                let gid = self.font.lookup(cell.ch as u32).unwrap();
                let glyf = self
                    .font
                    .parse_gid(gid as u16)
                    .unwrap()
                    .clone()
                    .unwrap()
                    .get_header();
                let col = index % self.row_size;
                let row = (index - col) / self.row_size;
                let x_cell = col as f32 * self.cell_metrics.width;
                let y_cell = row as f32 * self.cell_metrics.height;
                let baseline_x = x_cell;
                let baseline_y = y_cell + self.cell_metrics.baseline;
                let x0 = baseline_x + glyf.x_min as f32 * self.cell_metrics.scale;
                let y0 = baseline_y + glyf.y_max as f32 * self.cell_metrics.scale;
                let x1 = baseline_x + glyf.x_max as f32 * self.cell_metrics.scale;
                let y1 = baseline_y + glyf.y_min as f32 * self.cell_metrics.scale;
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
                let vertex_size = size_of::<Vertex>();
                // Calculates the offset in buffer memory
                let offset_write_index = vertex_size * 4 * index;
                vertex_buffer.write::<u32, _>(
                    offset_write_index,
                    &self.mesh.vertices[init_index..init_index + 4],
                );
            }
        }
    }
}

impl Handler for Screen {
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
