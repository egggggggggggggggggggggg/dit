use std::collections::HashSet;
pub mod rewrite;
pub mod ring_buf;
use atlas_gen::{allocator::ShelfAllocator, atlas::Atlas};
use font_parser::TtfFont;
use image::Rgb;

use crate::{
    ansii::{Handler, details::Attributes, utf_decoder::Utf8Decoder},
    renderer::{Mesh, shader::Vertex},
};

#[derive(Debug, Default, Clone)]
pub struct Cursor {
    pub y: usize,
    pub col: usize,
    pub visible: bool,
    pub blinking: bool,
}
#[derive(Clone)]
pub struct Cell {
    pub ch: char,
    pub cell_attr: Attributes,
}
impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            cell_attr: Attributes::default(),
        }
    }
}
#[derive(Debug)]
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
// defines a buffer write and sees if
#[derive(Debug, Clone)]
pub struct Range {
    pub start: usize,
    pub end: usize, // exclusive
}
fn merge_ranges(mut ranges: Vec<Range>) -> Vec<Range> {
    if ranges.is_empty() {
        return vec![];
    }

    // Sort by start
    ranges.sort_by_key(|r| r.start);

    let mut merged = Vec::new();
    let mut current = ranges[0].clone();

    for range in ranges.into_iter().skip(1) {
        // Merge if overlapping OR contiguous
        if range.start <= current.end + 1 {
            current.end = current.end.max(range.end);
        } else {
            merged.push(current);
            current = range;
        }
    }

    merged.push(current);
    merged
}
impl From<Range> for ash::vk::BufferCopy {
    fn from(value: Range) -> Self {
        Self {
            dst_offset: value.start as u64,
            src_offset: value.start as u64,
            size: (value.end - value.start) as u64,
        }
    }
}
pub struct Screen {
    pub cells: Vec<Cell>,
    pub cursor: Cursor,
    pub y_size: usize,
    pub col_size: usize,
    pub accumulator: Utf8Decoder,
    font: TtfFont,
    cell_metrics: CellMetrics,
    // theese sohuld be sorted
    dirty_cells: HashSet<usize>,
    atlas: Atlas<char, Rgb<u8>, ShelfAllocator>,
    pub mesh: Mesh,
}
// An arbitrary character for monospace fonts
const PLACEHOLDER: char = 'a';
#[inline(always)]
fn calculate_dims(
    logical_screen_size: winit::dpi::LogicalSize<f32>,
    cell_metrics: &CellMetrics,
) -> (usize, usize) {
    let y_size = (logical_screen_size.height / cell_metrics.height).floor() as usize;
    let col_size = (logical_screen_size.width / cell_metrics.width).floor() as usize;
    return (y_size, col_size);
}
impl Screen {
    // for the cursor that could be implemented via a static quad and controlled
    //via a ubo that specifies properties sorta like how instancing works
    // eg positioin and then the specific inherent properties
    // issue is how to write the shader to allow for this
    pub fn new(
        font_size: f32,
        font: TtfFont,
        texture_atlas: Atlas<char, Rgb<u8>, ShelfAllocator>,
        logical_screen_size: winit::dpi::LogicalSize<f32>,
    ) -> Self {
        let cell_metrics = CellMetrics::new(font_size, &font);
        let (y_size, col_size) = calculate_dims(logical_screen_size, &cell_metrics);
        let cells = vec![Cell::default(); y_size * col_size];
        Self {
            cells,
            cursor: Cursor::default(),
            y_size,
            col_size,
            accumulator: Utf8Decoder::new(),
            font,
            cell_metrics,
            dirty_cells: HashSet::new(),
            atlas: texture_atlas,
            mesh: Mesh::default(),
        }
    }
    pub fn resize(
        &mut self,
        new_font_size: f32,
        logical_screen_size: winit::dpi::LogicalSize<f32>,
    ) {
        if new_font_size != self.cell_metrics.font_size {
            // recalculate cell_metrics if font_size changes
            self.cell_metrics = CellMetrics::new(new_font_size, &self.font)
        }
        let (new_y_size, new_col_size) = calculate_dims(logical_screen_size, &self.cell_metrics);
        self.col_size = new_col_size;
        self.y_size = new_y_size;
        self.cells
            .resize_with(new_col_size * new_y_size, Default::default);
        self.construct_mesh();
        // must reconstruct the mesh from scratch
    }
    pub fn construct_mesh(&mut self) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut index_offset = 0u32;
        let mut y_cell = 0.0;
        for y in 0..self.y_size {
            let mut x_cell = 0.0;
            let y_start = y * self.col_size;
            let y_slice = &self.cells[y_start..y_start + self.col_size];
            for cell in y_slice {
                let baseline_x = x_cell;
                let baseline_y = 1080.0 - (y_cell + self.cell_metrics.baseline);
                let x0;
                let y0;
                let x1;
                let y1;
                let (u0, v0, u1, v1);
                if cell.ch.is_whitespace() {
                    // Use full cell bounds for whitespace quad
                    x0 = baseline_x;
                    y0 = y_cell;
                    x1 = baseline_x + self.cell_metrics.width;
                    y1 = y_cell + self.cell_metrics.height;
                    // Zero UVs
                    u0 = 0.0;
                    v0 = 0.0;
                    u1 = 0.0;
                    v1 = 0.0;
                } else {
                    // Safe glyph lookup (fallback to empty quad if missing)
                    if let Some(gid) = self.font.lookup(cell.ch as u32) {
                        if let Ok(Some(glyph)) = self.font.parse_gid(gid as u16) {
                            let header = glyph.get_header();

                            x0 = baseline_x + header.x_min as f32 * self.cell_metrics.scale;
                            y0 = baseline_y + header.y_max as f32 * self.cell_metrics.scale;
                            x1 = baseline_x + header.x_max as f32 * self.cell_metrics.scale;
                            y1 = baseline_y + header.y_min as f32 * self.cell_metrics.scale;

                            ([u0, v0], [u1, v1]) = self.atlas.get_uv(cell.ch);
                        } else {
                            // Failed parse → fallback to empty quad
                            x0 = baseline_x;
                            y0 = y_cell;
                            x1 = baseline_x + self.cell_metrics.width;
                            y1 = y_cell + self.cell_metrics.height;

                            u0 = 0.0;
                            v0 = 0.0;
                            u1 = 0.0;
                            v1 = 0.0;
                        }
                    } else {
                        // Missing glyph → fallback to empty quad
                        x0 = baseline_x;
                        y0 = y_cell;
                        x1 = baseline_x + self.cell_metrics.width;
                        y1 = y_cell + self.cell_metrics.height;

                        u0 = 0.0;
                        v0 = 0.0;
                        u1 = 0.0;
                        v1 = 0.0;
                    }
                }
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
                indices.extend_from_slice(&[
                    index_offset,
                    index_offset + 1,
                    index_offset + 2,
                    index_offset + 2,
                    index_offset + 3,
                    index_offset,
                ]);
                index_offset += 4;
                x_cell += self.cell_metrics.width;
            }

            y_cell += self.cell_metrics.height;
        }
        self.mesh.indices = indices;
        self.mesh.vertices = vertices;
    }
    // This is called by the update method in the application
    // Returns a vec of ranges of memory to be updated
    // For reconstruction of the entire mesh that just calls construct mesh
    // When the app calls resize it must then use the
    // mesh stored in screen instead of relying on diffs
    pub fn update_mesh(&mut self) -> Option<Vec<Range>> {
        if self.dirty_cells.is_empty() {
            return None;
        }
        let mut ranges = Vec::new();

        // Theoretically the index buffer shouldn't need updating unless its rezising at which point
        // Just remake the whole mesh
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
                let col = index % self.col_size;
                let y = index / self.col_size;
                let x_cell = col as f32 * self.cell_metrics.width;
                let y_cell = y as f32 * self.cell_metrics.height;
                let baseline_x = x_cell;
                let baseline_y = 1080.0 - (y_cell + self.cell_metrics.baseline);
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
                ranges.push(Range {
                    start: init_index,
                    end: init_index + 4,
                });
            }
        }
        self.dirty_cells.drain();
        Some(ranges)
    }
    pub fn write_char(&mut self, ch: char) {
        match ch {
            '\n' => {
                self.cursor.col = 0;
                self.cursor.y += 1;
            }
            '\r' => {
                self.cursor.col = 0;
            }
            '\x08' => {
                self.back_cursor(1);
            }
            c if !c.is_control() => {
                let index = self.cursor.y * self.y_size + self.cursor.col;

                if index < self.cells.len() {
                    self.cells[index].ch = c;
                    self.dirty_cells.insert(index);
                }

                self.advance_cursor(1);
            }

            _ => {}
        }
    }
    // advance the cursor by a given amount
    // will go to a new y if the first y is complete
    pub fn advance_cursor(&mut self, n: usize) {
        for _ in 0..n {
            self.cursor.col += 1;
            if self.cursor.col >= self.y_size {
                self.cursor.col = 0;
                self.cursor.y += 1;
            }
            if self.cursor.y * self.y_size >= self.cells.len() {
                break;
            }
        }
    }
    pub fn back_cursor(&mut self, n: i32) {
        for _ in 0..n {
            if self.cursor.col == 0 {
                if self.cursor.y == 0 {
                    return;
                }
                self.cursor.y -= 1;
                self.cursor.col = self.y_size - 1;
            } else {
                self.cursor.col -= 1;
            }

            let index = self.cursor.y * self.y_size + self.cursor.col;

            if index < self.cells.len() {
                self.cells[index].ch = ' ';
                self.dirty_cells.insert(index);
            }
        }
    }
}

impl Handler for Screen {
    fn cursor_up(&mut self, n: u16) {
        self.cursor.y = self.cursor.y.saturating_sub(n as usize);
    }
    fn cursor_down(&mut self, n: u16) {
        let max_y = self.y_size - 1;
        self.cursor.y = (self.cursor.y + n as usize).min(max_y);
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
            self.write_char(ch);
        }
    }
    fn bell() {}
    fn char_attributes(&mut self, params: &smallvec::SmallVec<[u16; 8]>) {}
    fn csi() {}

    fn cursor_position(&mut self, new_x: u16, new_y: u16) {}
    fn device_status_report(&mut self, param: u16) {}
    fn execute(&mut self, ctl_seq: u8) {
        match ctl_seq {
            // BS - Backspace
            0x08 => {
                self.back_cursor(1);
            }

            // HT - Horizontal tab (commonly tab stops every 8 columns)
            0x09 => {
                let tab_stop = 8;
                let next_tab = ((self.cursor.col / tab_stop) + 1) * tab_stop;
                let advance = next_tab.saturating_sub(self.cursor.col);
                self.advance_cursor(advance);
            }

            // LF - Line Feed
            0x0A => {
                self.cursor.col = 0;
            }

            // CR - Carriage Return
            0x0D => {
                self.cursor.y += 1;
            }

            _ => {}
        }
    }
    fn handle_osc(&mut self, u: &Vec<u8>) {}
    fn next_line(&mut self) {}
    fn previous_line(&mut self) {}
}
