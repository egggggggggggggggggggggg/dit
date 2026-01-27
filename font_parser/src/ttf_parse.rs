use std::collections::HashMap;

use std::sync::Arc;

use crate::cursor::Cursor;
use crate::error::{Error, ReadError};
pub use crate::table::*;
use math::lalg::{BezierCurve, Transform, transform_curve};
use std::fs::File;
use std::io::Read;
pub struct TtfFont {
    data: Vec<u8>,
    pub tables: HashMap<[u8; 4], TableRecord>,
    pub head: Head,
    pub maxp: Maxp,
    pub cmap: Vec<CMapGroup>,
    pub glyf: Glyf,
}

impl TtfFont {
    pub fn new(path: &str) -> Result<Self, Error> {
        let mut data = match read_file(path) {
            Ok(data) => data,
            Err(e) => return Err(Error::Io(e)),
        };
        let mut cursor = Cursor::set(&mut data, 0);
        let tables = parse_header(&mut cursor)?;
        let head = Head::new(&data, &tables)?;
        let maxp = Maxp::new(&data, &tables)?;
        let offsets = parse_loca(
            &data,
            &tables,
            maxp.glyph_count as usize,
            head.index_to_loc_format,
        )?;
        let cmap = parse_cmap(&data, &tables)?;
        let glyf = Glyf::new(offsets, &tables);
        Ok(Self {
            data,
            tables,
            head,
            maxp,
            cmap,
            glyf,
        })
    }
    pub fn parse_required() {}
    pub fn parse_gid(&mut self, gid: GlyphId) -> Result<Option<&Arc<Glyph>>, Error> {
        let mut cursor = Cursor::set(&self.data, 0);
        self.glyf.parse_glyf_block(gid, &mut cursor)?;
        let res = self.glyf.get_glyf(gid);
        Ok(res)
    }
    pub fn lookup(&mut self, c: u32) -> Option<u32> {
        for g in &self.cmap {
            if g.start_char <= c && c <= g.end_char {
                return Some(g.start_glyph + (c - g.start_char));
            }
        }
        None
    }
    pub fn assemble_glyf(&mut self, gid: GlyphId) -> Result<Vec<Vec<BezierCurve>>, Error> {
        let glyph = self.parse_gid(gid).unwrap().unwrap();
        let mut stack = vec![(glyph, Transform::identity())];
        let mut contours = Vec::new();
        while let Some((branch, transform)) = stack.pop() {
            match branch.as_ref() {
                Glyph::Simple(simple) => {
                    for contour in &simple.contours {
                        let mut new_contour = Vec::with_capacity(contours.len());
                        for curve in contour {
                            new_contour.push(transform_curve(curve, transform))
                        }
                        contours.push(new_contour);
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
        Ok(contours)
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
