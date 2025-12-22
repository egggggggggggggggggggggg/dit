use std::{collections::HashMap, hash::Hash, sync::Arc};

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
    curves: Vec<Vec<BezierCurve>>,
}
#[derive(Debug, Clone)]
struct Composite {
    tmat: [f32; 4],
    references: Vec<Arc<Glyph>>,
}
#[derive(Debug, Clone)]
enum Glyph {
    Simple(Arc<Simple>),
    Composite(Arc<Composite>),
}
type GlyphCache = HashMap<u32, Glyph>;
pub fn parse_glyfs(glyph_cache: GlyphCache) {

    //parse_glyf_block only gets called when the cache cannot find the glyph inside
}
pub fn parse_glyf_block(cursor: &mut Cursor, offset: usize) -> Result<(), ParseError> {
    cursor.seek(offset)?;
    let number_of_contours = cursor.read_i16()?;
    let x_min = cursor.read_i16()?;
    let y_min = cursor.read_i16()?;
    let x_max = cursor.read_i16()?;
    let y_max = cursor.read_i16()?;
    let glyph_header = GlyphHeader {
        number_of_contours,
        x_min,
        y_min,
        x_max,
        y_max,
    };
    if number_of_contours >= 0 {
        let contours = parse_simple(cursor, number_of_contours)?;
        let glyph = Simple {
            header: glyph_header,
            curves: contours,
        };
    } else {
        println!("This is a COMPOSITE GLYPH SKIPPING");
        parse_composite(cursor);
    }
    Ok(())
}

pub fn parse_simple(
    cursor: &mut Cursor,
    contour_num: i16,
) -> Result<Vec<Vec<BezierCurve>>, ParseError> {
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
    let mut contours = Vec::new();
    for end in end_points {
        let mut contour = Vec::new();
        let end = end as usize;
        for i in start..end {
            contour.push(GlyphPoint {
                x: x_coordinates[i],
                y: y_coordinates[i],
                on_curve_point: flags[i].contains(SimpleGlyphFlags::ON_CURVE_POINT),
            })
        }
        contours.push(contour);
        start = end + 1;
    }
    let mut curves = Vec::new();
    for contour in contours {
        curves.push(curve_from_contour(&contour)?);
    }
    Ok(curves)
}
//define the case to be linear
//this case is easier to calculate and render
//rename this to something else instead
#[derive(Debug, Clone, Copy)]
enum BezierCurve {
    Linear(f32, f32),
    Quadratic(f32, f32, f32),
    Cubic(f32, f32, f32, f32),
}

//change this to vec2 struct later
fn curve_from_contour(contour: &[GlyphPoint]) -> Result<Vec<BezierCurve>, ParseError> {
    let mut bezier_curves = Vec::new();
    let mut i = 0;
    while i < contour.len() - 1 {
        let p0 = &contour[i];
        let p1 = &contour[i + 1];
        //extract the vector value fromt the contour points,
        if p0.on_curve_point && p1.on_curve_point {
            bezier_curves.push(BezierCurve::Linear(0.0, 0.0));
            i += 2;
        } else if i + 2 < contour.len() {
            let p2 = &contour[i + 2];
            if p0.on_curve_point && !p1.on_curve_point && p2.on_curve_point {
                bezier_curves.push(BezierCurve::Quadratic(0.0, 0.0, 0.0));
                i += 2;
            }
        } else {
            return Err(ParseError::UnexpectedEof);
            //throw an error (random for now)
        }
    }
    Ok(bezier_curves)
}
struct SimpleGlyph {
    header: GlyphHeader,
    contours: Vec<Vec<BezierCurve>>,
}

struct GlyphPoint {
    pub x: i16,
    pub y: i16,
    pub on_curve_point: bool,
}

fn group_contours() {}
fn parse_composite(cursor: &mut Cursor) {}
