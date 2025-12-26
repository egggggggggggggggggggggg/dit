use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

pub mod error;
pub mod maxp;
pub mod table;
pub mod utils;
use self::error::*;
use crate::parse::maxp::Maxp;
use crate::parse::table::cmap::{CMapGroup, parse_cmap};
use crate::parse::table::glyf::*;
use crate::parse::table::head::{Head, TableRecord};
use std::fs::File;
use std::io::Read;
use utils::Cursor;
type GlyphID = u16;
pub struct TtfFont {
    data: Vec<u8>,
    tables: HashMap<[u8; 4], TableRecord>,
    pub head: Head,
    pub maxp: Maxp,
    pub cmap: Vec<CMapGroup>,
    pub glyf: Glyf,
}
pub enum Tag {
    Loca,
    Cmap,
    Head,
}

impl TtfFont {
    //takes a string for now will switch to smth else
    pub fn new(path: &str) -> Result<Self, Error> {
        let mut data = match read_file(path) {
            Ok(data) => data,
            Err(_e) => return Err(Error::FileNotRead),
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
        println!("finished parsing the required tables");
        Ok(Self {
            data,
            tables,
            head,
            maxp,
            cmap,
            glyf,
        })
    }
    pub fn parse_gid(&mut self, gid: GlyphID) -> Result<Option<&Arc<Glyph>>, Error> {
        let mut cursor = Cursor::set(&self.data, 0);
        self.glyf.parse_glyf_block(gid, &mut cursor)?;
        println!("finished parsing block");
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
    pub fn assemble_glyf(&mut self, gid: GlyphID) -> Result<Vec<Vec<BezierCurve>>, Error> {
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

fn transform_curve(curve: &BezierCurve, t: Transform) -> BezierCurve {
    match *curve {
        BezierCurve::Linear(p0, p1) => BezierCurve::Linear(t.apply(p0), t.apply(p1)),
        BezierCurve::Quadratic(p0, p1, p2) => {
            BezierCurve::Quadratic(t.apply(p0), t.apply(p1), t.apply(p2))
        }
        BezierCurve::Cubic(p0, p1, p2, p3) => {
            BezierCurve::Cubic(t.apply(p0), t.apply(p1), t.apply(p2), t.apply(p3))
        }
    }
}

fn read_file(path: &str) -> std::io::Result<Vec<u8>> {
    let mut data = Vec::new();
    File::open(path)?.read_to_end(&mut data)?;
    Ok(data)
}
fn parse_loca(
    data: &[u8],
    table: &HashMap<[u8; 4], TableRecord>,
    glyph_count: usize,
    format: i16,
) -> Result<Vec<u32>, Error> {
    let rec = table.get(b"loca").ok_or(Error::Test)?;
    let mut cursor = Cursor::set(data, rec.table_offset);
    let mut offsets: Vec<u32> = Vec::new();
    match format {
        0 => {
            let count = glyph_count / 2;
            for _ in 0..count {
                let raw = cursor.read_u16()?;
                offsets.push((raw as u32) * 2);
            }
        }
        1 => {
            let count = glyph_count / 4;
            for _ in 0..count {
                let raw = cursor.read_u32()?;
                offsets.push(raw);
            }
        }
        _ => return Err(Error::Test),
    }
    Ok(offsets)
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
