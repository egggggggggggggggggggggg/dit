use std::collections::HashMap;

use crate::{cursor::Cursor, error::Error, table::TableRecord};
#[derive(Debug)]
pub struct Hmtx {
    hmetrics: Vec<HMetric>,
    left_side_bearings: Vec<i16>,
}
#[derive(Clone, Copy, Debug)]
pub struct HMetric {
    pub advance_width: u16,
    pub left_side_bearing: i16,
}
impl Hmtx {
    pub fn parse(
        data: &[u8],
        tables: &HashMap<[u8; 4], TableRecord>,
        number_of_long_hor_metrics: u16,
        num_glyphs: u16,
    ) -> Result<Self, Error> {
        let rec = tables.get(b"hmtx").ok_or(Error::MissingTable("hmtx"))?;
        let mut cursor = Cursor::set(data, rec.table_offset);
        let mut hmetrics = Vec::with_capacity(number_of_long_hor_metrics as usize);
        for _ in 0..number_of_long_hor_metrics {
            hmetrics.push(HMetric {
                advance_width: cursor.read_u16()?,
                left_side_bearing: cursor.read_i16()?,
            });
        }
        let num_short_metrics = num_glyphs - number_of_long_hor_metrics;
        let mut left_side_bearings = Vec::with_capacity(num_short_metrics as usize);
        for _ in 0..num_short_metrics {
            left_side_bearings.push(cursor.read_i16()?);
        }
        Ok(Self {
            hmetrics,
            left_side_bearings,
        })
    }
    pub fn metric_for_glyph(&self, glyph_id: u16) -> HMetric {
        if (glyph_id as usize) < self.hmetrics.len() {
            self.hmetrics[glyph_id as usize]
        } else {
            let last = self.hmetrics.last().unwrap();
            HMetric {
                advance_width: last.advance_width,
                left_side_bearing: self.left_side_bearings
                    [(glyph_id as usize) - self.hmetrics.len()],
            }
        }
    }
}
