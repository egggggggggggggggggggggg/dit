use std::collections::HashMap;

use crate::{TableRecord, cursor::Cursor, error::Error};
#[derive(Debug, Clone)]
pub struct Post {
    pub italic_angle: f32,
    pub underline_position: i16,
    pub underline_thickness: i16,
    pub is_fixed_pitch: u32,
}
impl Post {
    pub fn parse(data: &[u8], tables: &HashMap<[u8; 4], TableRecord>) -> Result<Self, Error> {
        let rec = tables.get(b"post").ok_or(Error::MissingTable("post"))?;
        let mut cursor = Cursor::set(data, rec.table_offset);
        let _major = cursor.read_u16()?;
        let _minor = cursor.read_u16()?;
        let italic_angle = cursor.read_i32()? as f32 / 65536.0;
        let underline_position = cursor.read_i16()?;
        let underline_thickness = cursor.read_i16()?;
        let is_fixed_pitch = cursor.read_u32()?;
        Ok(Self {
            italic_angle,
            underline_position,
            underline_thickness,
            is_fixed_pitch,
        })
    }
}
