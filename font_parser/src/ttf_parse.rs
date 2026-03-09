use std::collections::HashMap;

use std::sync::Arc;

use crate::cursor::Cursor;
use crate::error::{Error, ReadError};
pub use crate::table::*;
use math::bezier::Bounds;
use math::contour::Contour;
use math::lalg::{Transform, transform_curve};
use math::shape::Shape;
use std::fs::File;
use std::io::Read;
#[derive(Debug, Clone)]
pub struct TtfFont {
    data: Vec<u8>,
    pub tables: HashMap<[u8; 4], TableRecord>,
    pub head: Head,
    pub maxp: Maxp,
    pub cmap: Vec<CMapGroup>,
    pub glyf: Glyf,
    pub hhea: Hhea,
    pub hmtx: Hmtx,
    pub post: Post,
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

impl TtfFont {
    pub fn new(path: &str) -> Result<Self, Error> {
        let mut data = match read_file(path) {
            Ok(data) => data,
            Err(e) => return Err(Error::Io(e)),
        };
        let mut cursor = Cursor::set(&mut data, 0);
        let tables = parse_header(&mut cursor)?;
        let head = Head::parse(&data, &tables)?;
        let maxp = Maxp::parse(&data, &tables)?;
        let offsets = parse_loca(
            &data,
            &tables,
            maxp.num_glyphs as usize,
            head.index_to_loc_format,
        )?;
        let num_glyphs = maxp.num_glyphs;
        let hhea = Hhea::parse(&data, &tables)?;
        let hmtx = Hmtx::parse(&data, &tables, hhea.number_of_long_hor_metrics, num_glyphs)?;
        let cmap = parse_cmap(&data, &tables)?;
        let glyf = Glyf::new(offsets, &tables);
        let post = Post::parse(&data, &tables)?;
        Ok(Self {
            data,
            tables,
            head,
            maxp,
            cmap,
            glyf,
            hhea,
            hmtx,
            post,
        })
    }
    pub fn parse_required() {}
    pub fn parse_gid(&mut self, gid: GlyphId) -> Result<Option<&Arc<Glyph>>, Error> {
        let mut cursor = Cursor::set(&self.data, 0);
        self.glyf.parse_glyf_block(gid, &mut cursor)?;
        let res = self.glyf.get_glyf(gid);
        Ok(res)
    }
    pub fn lookup(&self, c: u32) -> Option<u32> {
        for g in &self.cmap {
            if g.start_char <= c && c <= g.end_char {
                return Some(g.start_glyph + (c - g.start_char));
            }
        }
        None
    }
    pub fn assemble_glyf(&mut self, gid: GlyphId) -> Result<Shape, Error> {
        let glyph = self.parse_gid(gid).unwrap().unwrap();
        let header = glyph.get_header();
        let mut stack = vec![(glyph, Transform::identity())];
        let mut contours = Vec::new();
        while let Some((branch, transform)) = stack.pop() {
            match branch.as_ref() {
                Glyph::Simple(simple) => {
                    for contour in &simple.contours {
                        let mut edges = Vec::with_capacity(contours.len());
                        for curve in contour {
                            edges.push(transform_curve(curve, transform))
                        }
                        let contour = Contour { edges };
                        contours.push(contour);
                    }
                }
                Glyph::Composite(composite) => {
                    for component in &composite.components {
                        stack.push((
                            &component.reference,
                            transform.combine(component.transform_data),
                        ));
                    }
                }
            }
        }
        Ok(Shape {
            contours,
            bounds: Bounds::new(
                header.x_min as f64,
                header.x_max as f64,
                header.y_min as f64,
                header.y_max as f64,
            ),
        })
    }
    pub fn get_glyf_header(&self, gid: u16) -> Option<&GlyphHeader> {
        if let Some(glyf) = self.glyf.get_glyf(gid) {
            return Some(glyf.get_header());
        } else {
            return None;
        }
    }
    //a is an arbitrary letter to get metrics
    //This method only works for monospace as it assumes all cells = same
    pub fn get_cell_metriscs(&self, font_size: f32) -> CellMetrics {
        let units_per_em = self.head.units_per_em;
        let gid = self.lookup('a' as u32).unwrap();
        let arbitrary_metrics = self.hmtx.metric_for_glyph(gid as u16);
        let cell_advance = arbitrary_metrics.advance_width;
        let cell_ascent = self.hhea.ascent;
        let cell_descent = self.hhea.descent;
        let cell_height = cell_ascent - cell_descent + self.hhea.line_gap;
        let scale = font_size / units_per_em as f32;
        let cell_width_px = cell_advance as f32 * scale;
        let cell_height_px = cell_height as f32 * scale;
        let baseline_y = cell_ascent as f32 * scale;
        let underline_offset_px = self.post.underline_position as f32 * scale;
        let underline_thickness = self.post.underline_thickness as f32 * scale;
        CellMetrics {
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
fn read_file(path: &str) -> std::io::Result<Vec<u8>> {
    let mut data = Vec::new();
    File::open(path)?.read_to_end(&mut data)?;
    Ok(data)
}
fn parse_header(cursor: &mut Cursor) -> Result<HashMap<[u8; 4], TableRecord>, ReadError> {
    let _sfnt_version = cursor.read_u32()?;
    let num_tables = cursor.read_u16()?;
    let _search_range = cursor.read_u16()?;
    let _entry_selector = cursor.read_u16()?;
    let _range_shift = cursor.read_u16()?;
    let mut table_map = HashMap::new();
    for _ in 0..num_tables {
        let tag = cursor.read_u32()?.to_be_bytes();
        let checksum = cursor.read_u32()?;
        let table_offset = cursor.read_u32()? as usize;
        let length = cursor.read_u32()? as usize;
        let table_info = TableRecord {
            checksum,
            table_offset,
            length,
        };
        table_map.insert(tag, table_info);
    }
    Ok(table_map)
}
