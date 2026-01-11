use std::collections::HashMap;

use crate::{cursor::Cursor, error::Error, table::TableRecord};

pub fn parse_loca(
    data: &[u8],
    table: &HashMap<[u8; 4], TableRecord>,
    glyph_count: usize,
    format: i16,
) -> Result<Vec<u32>, Error> {
    let rec = table.get(b"loca").ok_or(Error::MissingTable("loca"))?;
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
        _ => return Err(Error::InvalidFormat("")),
    }
    Ok(offsets)
}
