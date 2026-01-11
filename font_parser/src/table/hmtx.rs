use std::collections::HashMap;

use crate::{cursor::Cursor, error::Error, table::TableRecord};
pub struct Hmtx {}
pub struct HMetric {
    pub advance_width: u16,
    pub left_side_bearing: u16,
}
impl Hmtx {
    pub fn parse(
        data: &[u8],
        table: HashMap<[u8; 4], TableRecord>,
        number_of_long_hor_metrics: u16,
    ) -> Result<Self, Error> {
        let rec = table.get(b"hmtx").ok_or(Error::MissingTable("hmtx"))?;
        let mut cursor = Cursor::set(data, rec.table_offset);
        let mut hmetrics = Vec::new();
        for _ in 0..number_of_long_hor_metrics {
            hmetrics.push(HMetric {
                advance_width: cursor.read_u16()?,
                left_side_bearing: cursor.read_u16()?,
            });
        }
        Err(Error::Unknown)
    }
}
