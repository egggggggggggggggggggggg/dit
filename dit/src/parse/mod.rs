use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut, Range};

pub mod error;
pub mod glyph;
pub mod maxp;
pub mod table;
pub mod utils;
use crate::parse::maxp::Maxp;
use crate::parse::table::cmap::{CMapGroup, parse_cmap};
use crate::parse::table::head::{Head, TableRecord};

use self::error::*;
use std::fs::File;
use std::io::Read;
use utils::Cursor;
type GlyphID = u32;

pub struct Glyph {}
pub struct GlyphCache {
    inner: HashMap<GlyphID, Glyph>,
}
impl GlyphCache {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }
}
impl Deref for GlyphCache {
    type Target = HashMap<GlyphID, Glyph>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl DerefMut for GlyphCache {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
pub struct TtfFont {
    data: Vec<u8>,
    tables: HashMap<[u8; 4], TableRecord>,
    head: Head,
    maxp: Maxp,
    offsets: Vec<u32>,
    cmap: Vec<CMapGroup>,
    glyph_cache: GlyphCache,
}
pub enum Tag {
    Loca,
    Cmap,
    Head,
}

impl TtfFont {
    //takes a string for now will switch to smth else
    pub fn new(path: &str) -> Result<(), Error> {
        let mut data = match read_file(path) {
            Ok(data) => data,
            Err(e) => return Err(Error::Test),
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
        let glyph_cache = GlyphCache::new();
        Self {
            data,
            tables,
            head,
            maxp,
            offsets,
            cmap,
            glyph_cache,
        };

        Ok(())
    }
    fn lookup(&mut self, c: u32) -> Option<u32> {
        for g in &self.cmap {
            if g.start_char <= c && c <= g.end_char {
                return Some(g.start_glyph + (c - g.start_char));
            }
        }
        None
    }
    ///parses a given unicode ranges glyph(any range works)
    pub fn glyf_unicode_range(a: Range<u32>) {
        for _ in a {
            //use the lookup function or smth here
        }
    }
    //looks up the glyph and its respective
    pub fn get_glyph(&mut self, gid: GlyphID) -> Result<&Glyph, Error> {
        if let Some(entry) = self.glyph_cache.get(&gid) {
            return Ok(entry);
        }
        let start = self.offsets[gid as usize];
        let end = self.offsets[gid as usize + 1];
        let length = end - start;
        if length == 0 {
            return Err(Error::Test);
        }

        Err(Error::Test)
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
