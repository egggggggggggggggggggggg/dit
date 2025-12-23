use std::collections::HashMap;

use crate::parse::{Cursor, GlyphCache, error::Error};

//should probably own a slice into the data
struct Glyf<'a> {
    offsets: Vec<u32>,
    glyph_cache: GlyphCache,
    cursor: Cursor<'a>,
}
impl<'a> Glyf<'a> {
    fn new(data: &'a [u8], offsets: Vec<u32>) -> Self {
        Self {
            offsets,
            glyph_cache: GlyphCache::new(),
            cursor: Cursor::set(data, 0),
        }
    }
    fn parse_transform(&mut self) {}
    fn parse_simple(&mut self) {}
    fn parse_composite(&mut self) {}
    fn parse_glyf_block(cursor: &mut Cursor, offset: usize) -> Result<(), Error> {
        let contour_count = cursor.read_i16()?;
        let x_min = cursor.read_i16()?;
        let y_min = cursor.read_i16()?;
        let x_max = cursor.read_i16()?;
        let x_min = cursor.read_i16()?;
        let y_min = cursor.read_i16()?;
        let x_max = cursor.read_i16()?;
        let y_max = cursor.read_i16()?;
        let y_max = cursor.read_i16()?;
        if contour_count >= 0 {
            let contours = parse_simple(cursor, contour_count);
        }
        Ok(())
    }
}

fn parse_glyf_block(
    cursor: &mut Cursor,
    offset: usize,
    glyph_cache: GlyphCache,
) -> Result<(), Error> {
    let contour_count = cursor.read_i16()?;
    let x_min = cursor.read_i16()?;
    let y_min = cursor.read_i16()?;
    let x_max = cursor.read_i16()?;
    let x_min = cursor.read_i16()?;
    let y_min = cursor.read_i16()?;
    let x_max = cursor.read_i16()?;
    let y_max = cursor.read_i16()?;
    let y_max = cursor.read_i16()?;
    if contour_count >= 0 {
        let contours = parse_simple(cursor, contour_count);
    }
    Ok(())
}
fn parse_simple(cursor: &mut Cursor, contour_count: i16) -> Result<(), Error> {
    Ok(())
}
fn parse_composite(cursor: &mut Cursor) {
    let mut stack = Vec::new();
    while let Some(item) = stack.pop() {
        //
        //resolve the queue
    }
}
