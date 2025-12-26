use std::collections::HashMap;

use crate::parse::{error::Error, utils::Cursor};
const MAGIC_NUMBER: u32 = 0x5F0F3CF5;
#[derive(Debug)]
pub struct TableRecord {
    pub checksum: u32,
    pub table_offset: usize,
    pub length: usize,
}
#[derive(Debug)]
pub struct Head {
    pub major: u16,
    pub minor: u16,
    pub font_revision: f32,
    pub checksum: u32,
    pub magic_number: u32,
    pub flags: u16,
    pub units_per_em: u16,
    pub created: i64,
    pub modified: i64,
    pub x_min: i16,
    pub y_min: i16,
    pub x_max: i16,
    pub y_max: i16,
    pub mac_style: u16,
    pub lowest_rec_ppem: u16,
    pub font_direction_hint: i16,
    pub index_to_loc_format: i16,
    pub glyph_data_format: i16,
}
impl Head {
    pub fn new(data: &[u8], tables: &HashMap<[u8; 4], TableRecord>) -> Result<Self, Error> {
        let rec = tables.get(b"head").ok_or(Error::Test)?;
        let mut cursor = Cursor::set(data, rec.table_offset);
        let major = cursor.read_u16()?;
        let minor = cursor.read_u16()?;
        let font_rev_major = cursor.read_u16()? as u32;
        let font_rev_minor = cursor.read_u16()? as u32;
        let font_revision = (font_rev_minor << 16 | font_rev_major) as f32 / (1 << 16) as f32;
        let checksum = cursor.read_u32()?;
        let magic_number = cursor.read_u32()?;
        if magic_number != MAGIC_NUMBER {
            return Err(Error::Test);
        }
        let flags = cursor.read_u16()?;
        let units_per_em = cursor.read_u16()?;
        let created = cursor.read_i64()?;
        let modified = cursor.read_i64()?;
        let x_min = cursor.read_i16()?;
        let y_min = cursor.read_i16()?;
        let x_max = cursor.read_i16()?;
        let y_max = cursor.read_i16()?;
        let mac_style = cursor.read_u16()?;
        let lowest_rec_ppem = cursor.read_u16()?;
        let font_direction_hint = cursor.read_i16()?;
        let index_to_loc_format = cursor.read_i16()?;
        let glyph_data_format = cursor.read_i16()?;
        Ok(Head {
            major,
            minor,
            font_revision,
            checksum,
            magic_number,
            flags,
            units_per_em,
            created,
            modified,
            x_min,
            y_min,
            x_max,
            y_max,
            mac_style,
            lowest_rec_ppem,
            font_direction_hint,
            index_to_loc_format,
            glyph_data_format,
        })
    }
}
