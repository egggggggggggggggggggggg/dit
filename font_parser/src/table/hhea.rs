use std::collections::HashMap;

use crate::{cursor::Cursor, error::Error, table::TableRecord};
#[derive(Debug, Clone)]
pub struct Hhea {
    pub major: u16,
    pub minor: u16,
    pub ascent: i16,
    pub descent: i16,
    pub line_gap: i16,
    pub advance_width_max: u16,
    pub min_left_side_bearing: i16,
    pub min_right_side_bearing: i16,
    pub x_max_extent: i16,
    pub caret_slope_rise: i16,
    pub caret_slope_run: i16,
    pub caret_offset: i16,
    pub metric_data_format: i16,
    pub number_of_long_hor_metrics: u16,
}
impl Hhea {
    pub fn parse(data: &[u8], tables: &HashMap<[u8; 4], TableRecord>) -> Result<Self, Error> {
        let rec = tables.get(b"hhea").ok_or(Error::MissingTable("hhea"))?;
        let mut cursor = Cursor::set(data, rec.table_offset);
        let major = cursor.read_u16()?;
        let minor = cursor.read_u16()?;
        let ascent = cursor.read_i16()?;
        let descent = cursor.read_i16()?;
        let line_gap = cursor.read_i16()?;
        let advance_width_max = cursor.read_u16()?;
        let min_left_side_bearing = cursor.read_i16()?;
        let min_right_side_bearing = cursor.read_i16()?;
        let x_max_extent = cursor.read_i16()?;
        let caret_slope_rise = cursor.read_i16()?;
        let caret_slope_run = cursor.read_i16()?;
        let caret_offset = cursor.read_i16()?;
        cursor.seek(cursor.position() + 8)?;
        //Reserved pads
        let metric_data_format = cursor.read_i16()?;
        let number_of_long_hor_metrics = cursor.read_u16()?;
        Ok(Self {
            major,
            minor,
            ascent,
            descent,
            line_gap,
            advance_width_max,
            min_left_side_bearing,
            min_right_side_bearing,
            x_max_extent,
            caret_slope_rise,
            caret_slope_run,
            caret_offset,
            metric_data_format,
            number_of_long_hor_metrics,
        })
    }
}
