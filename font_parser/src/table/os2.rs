use std::collections::HashMap;

use crate::{TableRecord, cursor::Cursor, error::Error};

#[derive(Debug)]
pub struct Os2Table {
    pub version: u16,
    pub x_avg_char_width: i16,
    pub us_weight_class: u16,
    pub us_width_class: u16,
    pub fs_type: u16,

    pub y_subscript_x_size: i16,
    pub y_subscript_y_size: i16,
    pub y_subscript_x_offset: i16,
    pub y_subscript_y_offset: i16,

    pub y_superscript_x_size: i16,
    pub y_superscript_y_size: i16,
    pub y_superscript_x_offset: i16,
    pub y_superscript_y_offset: i16,

    pub y_strikeout_size: i16,
    pub y_strikeout_position: i16,

    pub s_family_class: i16,
    pub panose: [u8; 10],

    pub ul_unicode_range1: u32,
    pub ul_unicode_range2: u32,
    pub ul_unicode_range3: u32,
    pub ul_unicode_range4: u32,

    pub ach_vend_id: [u8; 4],
    pub fs_selection: u16,
    pub us_first_char_index: u16,
    pub us_last_char_index: u16,

    pub s_typo_ascender: i16,
    pub s_typo_descender: i16,
    pub s_typo_line_gap: i16,

    pub us_win_ascent: u16,
    pub us_win_descent: u16,

    // Version >= 1
    pub ul_code_page_range1: Option<u32>,
    pub ul_code_page_range2: Option<u32>,

    // Version >= 2
    pub sx_height: Option<i16>,
    pub s_cap_height: Option<i16>,
    pub us_default_char: Option<u16>,
    pub us_break_char: Option<u16>,
    pub us_max_context: Option<u16>,

    // Version >= 5
    pub us_lower_optical_point_size: Option<u16>,
    pub us_upper_optical_point_size: Option<u16>,
}
impl Os2Table {
    pub fn parse(data: &[u8], tables: &HashMap<[u8; 4], TableRecord>) -> Result<Self, Error> {
        let rec = tables.get(b"OS/2").ok_or(Error::MissingTable("post"))?;
        let mut cursor = Cursor::set(data, rec.table_offset);
        let version = cursor.read_u16()?;

        let x_avg_char_width = cursor.read_i16()?;
        let us_weight_class = cursor.read_u16()?;
        let us_width_class = cursor.read_u16()?;
        let fs_type = cursor.read_u16()?;

        let y_subscript_x_size = cursor.read_i16()?;
        let y_subscript_y_size = cursor.read_i16()?;
        let y_subscript_x_offset = cursor.read_i16()?;
        let y_subscript_y_offset = cursor.read_i16()?;

        let y_superscript_x_size = cursor.read_i16()?;
        let y_superscript_y_size = cursor.read_i16()?;
        let y_superscript_x_offset = cursor.read_i16()?;
        let y_superscript_y_offset = cursor.read_i16()?;

        let y_strikeout_size = cursor.read_i16()?;
        let y_strikeout_position = cursor.read_i16()?;

        let s_family_class = cursor.read_i16()?;

        let mut panose = [0u8; 10];
        for i in 0..10 {
            panose[i] = cursor.read_u8()?;
        }

        let ul_unicode_range1 = cursor.read_u32()?;
        let ul_unicode_range2 = cursor.read_u32()?;
        let ul_unicode_range3 = cursor.read_u32()?;
        let ul_unicode_range4 = cursor.read_u32()?;

        let mut ach_vend_id = [0u8; 4];
        for i in 0..4 {
            ach_vend_id[i] = cursor.read_u8()?;
        }

        let fs_selection = cursor.read_u16()?;
        let us_first_char_index = cursor.read_u16()?;
        let us_last_char_index = cursor.read_u16()?;

        let s_typo_ascender = cursor.read_i16()?;
        let s_typo_descender = cursor.read_i16()?;
        let s_typo_line_gap = cursor.read_i16()?;

        let us_win_ascent = cursor.read_u16()?;
        let us_win_descent = cursor.read_u16()?;

        // Version >= 1
        let (ul_code_page_range1, ul_code_page_range2) = if version >= 1 {
            (Some(cursor.read_u32()?), Some(cursor.read_u32()?))
        } else {
            (None, None)
        };

        // Version >= 2
        let (sx_height, s_cap_height, us_default_char, us_break_char, us_max_context) =
            if version >= 2 {
                (
                    Some(cursor.read_i16()?),
                    Some(cursor.read_i16()?),
                    Some(cursor.read_u16()?),
                    Some(cursor.read_u16()?),
                    Some(cursor.read_u16()?),
                )
            } else {
                (None, None, None, None, None)
            };

        // Version >= 5
        let (us_lower_optical_point_size, us_upper_optical_point_size) = if version >= 5 {
            (Some(cursor.read_u16()?), Some(cursor.read_u16()?))
        } else {
            (None, None)
        };

        Ok(Self {
            version,
            x_avg_char_width,
            us_weight_class,
            us_width_class,
            fs_type,

            y_subscript_x_size,
            y_subscript_y_size,
            y_subscript_x_offset,
            y_subscript_y_offset,

            y_superscript_x_size,
            y_superscript_y_size,
            y_superscript_x_offset,
            y_superscript_y_offset,

            y_strikeout_size,
            y_strikeout_position,

            s_family_class,
            panose,

            ul_unicode_range1,
            ul_unicode_range2,
            ul_unicode_range3,
            ul_unicode_range4,

            ach_vend_id,
            fs_selection,
            us_first_char_index,
            us_last_char_index,

            s_typo_ascender,
            s_typo_descender,
            s_typo_line_gap,

            us_win_ascent,
            us_win_descent,

            ul_code_page_range1,
            ul_code_page_range2,

            sx_height,
            s_cap_height,
            us_default_char,
            us_break_char,
            us_max_context,

            us_lower_optical_point_size,
            us_upper_optical_point_size,
        })
    }
}
