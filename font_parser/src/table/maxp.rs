use std::collections::HashMap;

use crate::{cursor::Cursor, error::Error, table::TableRecord};

#[derive(Debug)]
pub struct Maxp {
    pub vers_major: u32,
    pub vers_minor: u32,
    pub num_glyphs: u16,
    pub max_points: u16,
    pub max_contours: u16,
    pub max_composite_points: u16,
    pub max_composite_contours: u16,
    pub max_zones: u16,
    pub max_twilight_points: u16,
    pub max_storage: u16,
    pub max_function_defs: u16,
    pub max_stack_elements: u16,
    pub max_size_of_instructions: u16,
    pub max_component_elements: u16,
    pub max_component_depth: u16,
}
impl Maxp {
    pub fn parse(data: &[u8], tables: &HashMap<[u8; 4], TableRecord>) -> Result<Self, Error> {
        let rec = tables.get(b"maxp").ok_or(Error::MissingTable("maxp"))?;
        let mut cursor = Cursor::set(data, rec.table_offset);
        let vers_major = cursor.read_u16()? as u32;
        let vers_minor = cursor.read_u16()? as u32;
        let num_glyphs = cursor.read_u16()?;
        let max_points = cursor.read_u16()?;
        let max_contours = cursor.read_u16()?;
        let max_composite_points = cursor.read_u16()?;
        let max_composite_contours = cursor.read_u16()?;
        let max_zones = cursor.read_u16()?;
        let max_twilight_points = cursor.read_u16()?;
        let max_storage = cursor.read_u16()?;
        let max_function_defs = cursor.read_u16()?;
        let max_stack_elements = cursor.read_u16()?;
        let max_size_of_instructions = cursor.read_u16()?;
        let max_component_elements = cursor.read_u16()?;
        let max_component_depth = cursor.read_u16()?;
        Ok(Self {
            vers_major,
            vers_minor,
            num_glyphs,
            max_points,
            max_contours,
            max_composite_points,
            max_composite_contours,
            max_zones,
            max_twilight_points,
            max_storage,
            max_function_defs,
            max_stack_elements,
            max_size_of_instructions,
            max_component_elements,
            max_component_depth,
        })
    }
}
