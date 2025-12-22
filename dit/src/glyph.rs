use crate::{err::ParseError, read_utils::Cursor};
use bitflags::bitflags;
//replace bitflags with own implementation. dont need the whole crate
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SimpleGlyphFlags: u8 {
        const ON_CURVE_POINT                       = 0x01;
        const X_SHORT_VECTOR                       = 0x02;
        const Y_SHORT_VECTOR                       = 0x04;
        const REPEAT_FLAG                          = 0x08;
        const X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR = 0x10;
        const Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR = 0x20;
        const OVERLAP_SIMPLE                       = 0x40;
        const Reserved                             = 0x80;
    }
}
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ComponentFlags: u16 {
        const ARG_1_AND_2_ARE_WORDS        = 0x0001;
        const ARGS_ARE_XY_VALUES           = 0x0002;
        const ROUND_XY_TO_GRID             = 0x0004;
        const WE_HAVE_A_SCALE              = 0x0008;
        const MORE_COMPONENTS              = 0x0020;
        const WE_HAVE_AN_X_AND_Y_SCALE     = 0x0040;
        const WE_HAVE_A_TWO_BY_TWO         = 0x0080;
        const WE_HAVE_INSTRUCTIONS         = 0x0100;
        const USE_MY_METRICS               = 0x0200;
        const OVERLAP_COMPOUND             = 0x0400;
        const SCALED_COMPONENT_OFFSET      = 0x0800;
        const UNSCALED_COMPONENT_OFFSET    = 0x1000;
        const RESERVED                     = 0xE010;
    }
}

enum Glyph {
    Composite(Composite),
    Simple(Simple),
}
#[derive(Debug, Clone)]
struct GlyphHeader {
    number_of_contours: i16,
    x_min: i16,
    y_min: i16,
    x_max: i16,
    y_max: i16,
}
#[derive(Debug, Clone)]
struct Simple {
    header: GlyphHeader,
    end_points: Vec<u16>,
}
struct Composite {}
pub fn parse_glyf_block(cursor: &mut Cursor, offset: usize) -> Result<(), ParseError> {
    cursor.seek(offset)?;
    let number_of_contours = cursor.read_i16()?;
    let x_min = cursor.read_i16()?;
    let y_min = cursor.read_i16()?;
    let x_max = cursor.read_i16()?;
    let y_max = cursor.read_i16()?;
    if number_of_contours >= 0 {
        parse_simple(cursor, number_of_contours);
    } else {
        println!("This is a COMPOSITE GLYPH SKIPPING");
        parse_composite(cursor);
    }
    Ok(())
}

pub fn parse_simple(cursor: &mut Cursor, contour_num: i16) -> Result<(), ParseError> {
    let mut end_points = Vec::new();
    println!("contour_num: {}", contour_num);
    for _ in 0..contour_num {
        end_points.push(cursor.read_u16()?);
    }
    let instruction_length = cursor.read_u16()?;
    let mut instructions = Vec::new();
    for _ in 0..instruction_length {
        instructions.push(cursor.read_u8()?);
    }
    let mut flags = Vec::new();
    let mut x_coordinates = Vec::new();
    let mut y_coordinates = Vec::new();
    let num_points = end_points[contour_num as usize - 1] + 1;
    let mut i = 0;
    while i < num_points {
        let flag_byte = cursor.read_u8()?;
        let flag = SimpleGlyphFlags::from_bits_truncate(flag_byte);
        if flag.contains(SimpleGlyphFlags::REPEAT_FLAG) {
            let repeat_count = cursor.read_u8()?;
            for _ in 0..=repeat_count {
                flags.push(flag);
                i += 1;
            }
        } else {
            flags.push(flag);
            i += 1;
        }
    }
    let mut current_x = 0;
    for flag in &flags {
        let dx = if flag.contains(SimpleGlyphFlags::X_SHORT_VECTOR) {
            let val = cursor.read_u8()? as i16;
            if flag.contains(SimpleGlyphFlags::X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR) {
                val
            } else {
                -val
            }
        } else {
            if flag.contains(SimpleGlyphFlags::X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR) {
                0
            } else {
                cursor.read_i16()?
            }
        };
        current_x += dx;
        x_coordinates.push(current_x);
    }
    let mut current_y: i16 = 0;

    for flag in &flags {
        let dy = if flag.contains(SimpleGlyphFlags::Y_SHORT_VECTOR) {
            let val = cursor.read_u8()? as i16;
            if flag.contains(SimpleGlyphFlags::Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR) {
                val
            } else {
                -val
            }
        } else {
            if flag.contains(SimpleGlyphFlags::Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR) {
                0
            } else {
                cursor.read_i16()?
            }
        };

        current_y += dy;
        y_coordinates.push(current_y);
    }
    println!(
        "x_coordinates: {:?} \ny_coordinates: {:?} \nend_points: {:?} \n flags: {:?} \n",
        x_coordinates, y_coordinates, end_points, flags,
    );
    let mut start = 0;
    for end in end_points {
        let end = end as usize;
    }
    Ok(())
}
struct GlyphPoint {
    pub x: i16,
    pub y: i16,
    pub on_curve_point: bool,
}

fn group_contours() {}
fn parse_composite(cursor: &mut Cursor) {}
