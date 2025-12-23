use crate::parse::{
    error::{Error, ParseError},
    table::cmap::CMapGroup,
    utils::Cursor,
};

pub fn parse_format4(cursor: &mut Cursor) -> Result<Vec<CMapGroup>, Error> {
    let format = cursor.read_u16()?;
    let length = cursor.read_u16()?;
    let language = cursor.read_u16()?;
    let segCountX2 = cursor.read_u16()?;
    let search_range = cursor.read_u16()?;
    let entry_selector = cursor.read_u16()?;
    let range_shift = cursor.read_u16()?;
    let mut end_codes = Vec::new();
    let mut start_codes = Vec::new();
    let mut id_deltas = Vec::new();
    let mut id_range_offset = Vec::new();
    let mut glyph_id_array = Vec::new();
    let seg_count = segCountX2 as usize / 2 as usize;
    for _ in 0..seg_count {
        end_codes.push(cursor.read_u16()?);
    }
    let reserved_pad = cursor.read_u16()?;
    for _ in 0..seg_count {
        start_codes.push(cursor.read_u16()?);
    }
    for _ in 0..seg_count {
        id_deltas.push(cursor.read_i16()?);
    }
    for _ in 0..seg_count {
        id_range_offset.push(cursor.read_u16()?);
    }
    let bytes_read = 16 + seg_count * 8;
    let remaining = length as usize - bytes_read;
    for _ in 0..(remaining / 2) {
        glyph_id_array.push(cursor.read_u16()?);
    }
    let mut groups = vec![];
    for i in 0..seg_count {
        let start = start_codes[i];
        let end = end_codes[i];
        if start == 0xFFFF && end == 0xFFFF {
            continue;
        }
        let delta = id_deltas[i] as i32;
        let range_offset = id_range_offset[i];
        let mut c = start as u32;
        while c <= end as u32 {
            let glyph = if range_offset == 0 {
                ((c as i32 + delta) & 0xFFFF) as u16
            } else {
                let roffset = (range_offset / 2) as usize;
                let idx = roffset + (c as usize - start as usize) + i - seg_count;
                if idx >= glyph_id_array.len() {
                    c += 1;
                    continue;
                }
                let raw = glyph_id_array[idx];
                if raw == 0 {
                    c += 1;
                    continue;
                }
                ((raw as i32 + delta) & 0xFFFF) as u16
            };
            if glyph == 0 {
                c += 1;
                continue;
            }
            let run_start_c = c;
            let run_start_g = glyph as u32;
            c += 1;
            while c <= end as u32 {
                let next = if range_offset == 0 {
                    ((c as i32 + delta) & 0xFFFF) as u16
                } else {
                    let roffset = (range_offset / 2) as usize;
                    let idx = roffset + (c as usize - start as usize) + i - seg_count;
                    if idx >= glyph_id_array.len() {
                        break;
                    }
                    let raw = glyph_id_array[idx];
                    if raw == 0 {
                        break;
                    }
                    ((raw as i32 + delta) & 0xFFFF) as u16
                };
                if next as u32 != run_start_g + (c - run_start_c) {
                    break;
                }
                c += 1;
            }
            groups.push(CMapGroup {
                start_char: run_start_c,
                end_char: c - 1,
                start_glyph: run_start_g,
            });
        }
    }
    Ok(groups)
}
