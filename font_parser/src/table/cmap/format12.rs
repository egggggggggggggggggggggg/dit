use crate::{cursor::Cursor, error::Error, table::cmap::CMapGroup};

pub fn parse_format12(cursor: &mut Cursor) -> Result<Vec<CMapGroup>, Error> {
    let format = cursor.read_u16()?;
    if format != 12 {
        return Err(Error::InvalidFormat("Format 12"));
    }
    let _reserved = cursor.read_u16()?;
    let _length = cursor.read_u32()?;
    let _language = cursor.read_u32()?;
    let num_groups = cursor.read_u32()?;
    let mut groups = Vec::with_capacity(num_groups as usize);
    for _ in 0..num_groups {
        groups.push(CMapGroup {
            start_char: cursor.read_u32()?,
            end_char: cursor.read_u32()?,
            start_glyph: cursor.read_u32()?,
        });
    }
    Ok(groups)
}
