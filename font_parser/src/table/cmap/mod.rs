use core::panic;
use std::collections::HashMap;

use crate::error::ParseError;
use crate::table::TableRecord;
use crate::{cursor::Cursor, error::Error};

pub mod format12;
pub mod format4;
use self::format4::*;
use self::format12::*;
#[derive(Debug, Clone)]
pub struct CMapGroup {
    pub start_char: u32,
    pub end_char: u32,
    pub start_glyph: u32,
}
pub struct CMapSubtable {
    pub platform_id: u16,
    pub encoding_id: u16,
    pub offset: usize,
    pub format: u16,
}
struct CMap {}

fn select_best_cmap(subtables: &[CMapSubtable]) -> Option<&CMapSubtable> {
    subtables
        .iter()
        .find(|s| s.platform_id == 0 && s.encoding_id == 4 && s.format == 12)
        .or_else(|| {
            subtables
                .iter()
                .find(|s| s.platform_id == 3 && s.encoding_id == 10 && s.format == 12)
        })
        .or_else(|| {
            subtables
                .iter()
                .find(|s| s.platform_id == 0 && s.encoding_id == 3 && s.format == 4)
        })
        .or_else(|| {
            subtables
                .iter()
                .find(|s| s.platform_id == 3 && s.encoding_id == 1 && s.format == 4)
        })
}
pub fn parse_cmap(
    data: &[u8],
    tables: &HashMap<[u8; 4], TableRecord>,
) -> Result<Vec<CMapGroup>, Error> {
    let rec = tables.get(b"cmap").ok_or(Error::MissingTable("cmap"))?;
    let mut cursor = Cursor::set(data, rec.table_offset);
    let version = cursor.read_u16()?;
    let num_tables = cursor.read_u16()?;
    let mut subtables = Vec::new();
    for _ in 0..num_tables {
        let platform_id = cursor.read_u16()?;
        let encoding_id = cursor.read_u16()?;
        let offset = cursor.read_u32()? as usize;
        let saved = cursor.position();
        cursor.seek(rec.table_offset + offset)?;
        let format = cursor.read_u16()?;
        cursor.seek(saved)?;
        subtables.push(CMapSubtable {
            platform_id,
            encoding_id,
            offset,
            format,
        });
    }
    let chosen = select_best_cmap(&subtables).ok_or(ParseError::InvalidCmap)?;
    cursor.seek(rec.table_offset + chosen.offset)?;
    match chosen.format {
        4 => parse_format4(&mut cursor),
        12 => parse_format12(&mut cursor),
        _ => panic!(),
    }
}
pub fn lookup(groups: &[CMapGroup], c: u32) -> Option<u32> {
    for g in groups {
        if g.start_char <= c && c <= g.end_char {
            return Some(g.start_glyph + (c - g.start_char));
        }
    }
    None
}
