use crate::{
    cursor::Cursor,
    error::Error,
    table::{GlyphId, TableRecord},
};
use math::{
    bezier::{BezierTypes, EdgeColor, LinearBezier, QuadraticBezier},
    lalg::{BezierCurve, Transform, Vec2},
};
use std::{collections::HashMap, sync::Arc};
#[derive(Copy, Clone, Debug)]
enum ComponentFlags {
    ARG_1_AND_2_ARE_WORDS = 0x0001,
    ARGS_ARE_XY_VALUES = 0x0002,
    ROUND_XY_TO_GRID = 0x0004,
    WE_HAVE_A_SCALE = 0x0008,
    MORE_COMPONENTS = 0x0020,
    WE_HAVE_AN_X_AND_Y_SCALE = 0x0040,
    WE_HAVE_A_TWO_BY_TWO = 0x0080,
    WE_HAVE_INSTRUCTIONS = 0x0100,
    USE_MY_METRICS = 0x0200,
    OVERLAP_COMPOUND = 0x0400,
    SCALED_COMPONENT_OFFSET = 0x0800,
    UNSCALED_COMPONENT_OFFSET = 0x1000,
    RESERVED = 0xE010,
}
#[derive(Copy, Clone, Debug)]
enum SimpleFlags {
    ON_CURVE_POINT = 0x01,
    X_SHORT_VECTOR = 0x02,
    Y_SHORT_VECTOR = 0x04,
    REPEAT_FLAG = 0x08,
    X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR = 0x10,
    Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR = 0x20,
    OVERLAP_SIMPLE = 0x40,
    RESERVED = 0x80,
}
pub type GlyphCache = HashMap<GlyphId, Arc<Glyph>>;

#[derive(Debug)]
pub struct SimpleGlyph {
    pub header: GlyphHeader,
    pub contours: Vec<Vec<BezierTypes>>,
}
#[derive(Debug)]
pub struct CompositeGlyph {
    pub components: Vec<Component>,
    pub header: GlyphHeader,
}
#[derive(Debug, Clone, Copy)]
pub struct GlyphHeader {
    pub contour_count: i16,
    pub x_min: i16,
    pub y_min: i16,
    pub x_max: i16,
    pub y_max: i16,
}
#[derive(Debug)]
pub enum Glyph {
    Simple(SimpleGlyph),
    Composite(CompositeGlyph),
}
impl Glyph {
    pub fn get_header(&self) -> &GlyphHeader {
        match self {
            Self::Simple(simple) => &simple.header,
            Self::Composite(composite) => &composite.header,
        }
    }
}

struct UnresolvedComponent {
    flags: u16,
    gid: GlyphId,
    transform: Transform,
}
impl UnresolvedComponent {
    fn resolve(self, reference: Arc<Glyph>) -> Component {
        Component {
            flags: self.flags,
            gid: self.gid,
            reference: reference,
            transform_data: self.transform,
        }
    }
}
#[derive(Debug)]
pub struct Component {
    pub flags: u16,
    pub gid: u16,
    pub reference: Arc<Glyph>,
    pub transform_data: Transform,
}
#[derive(Debug, Clone)]
pub struct Glyf {
    offsets: Vec<u32>,
    glyph_cache: GlyphCache,
    local_offset: usize,
}
impl Glyf {
    pub fn new(offsets: Vec<u32>, record: &HashMap<[u8; 4], TableRecord>) -> Self {
        let local_offset = record
            .get(b"glyf")
            .ok_or(Error::MissingTable("glyf"))
            .unwrap()
            .table_offset;
        Self {
            offsets,
            glyph_cache: GlyphCache::default(),
            local_offset,
        }
    }
    fn parse_transform(&mut self, cursor: &mut Cursor, flags: u16) -> Result<Transform, Error> {
        let mut transform = Transform::identity();
        if flags & (ComponentFlags::WE_HAVE_A_SCALE as u16) != 0 {
            let scale = cursor.read_f2dot14()?.to_f32();
            transform.a = scale as f64;
            transform.d = scale as f64;
        } else if flags & (ComponentFlags::WE_HAVE_AN_X_AND_Y_SCALE as u16) != 0 {
            transform.a = cursor.read_f2dot14()?.to_f32() as f64;
            transform.d = cursor.read_f2dot14()?.to_f32() as f64;
        } else if flags & (ComponentFlags::WE_HAVE_A_TWO_BY_TWO as u16) != 0 {
            transform.a = cursor.read_f2dot14()?.to_f32() as f64;
            transform.b = cursor.read_f2dot14()?.to_f32() as f64;
            transform.c = cursor.read_f2dot14()?.to_f32() as f64;
            transform.d = cursor.read_f2dot14()?.to_f32() as f64;
        }
        Ok(transform)
    }
    fn parse_args(&mut self, cursor: &mut Cursor, flags: u16) -> Result<(i32, i32), Error> {
        let args = if flags & ComponentFlags::ARG_1_AND_2_ARE_WORDS as u16 != 0 {
            (cursor.read_i16()? as i32, cursor.read_i16()? as i32)
        } else if flags & ComponentFlags::ARGS_ARE_XY_VALUES as u16 != 0 {
            (cursor.read_i8()? as i32, cursor.read_i8()? as i32)
        } else {
            (cursor.read_u8()? as i32, cursor.read_u8()? as i32)
        };
        Ok(args)
    }
    pub fn get_glyf(&self, gid: GlyphId) -> Option<&Arc<Glyph>> {
        self.glyph_cache.get(&gid)
    }
    pub fn parse_glyf_block(&mut self, gid: GlyphId, cursor: &mut Cursor) -> Result<(), Error> {
        if self.glyph_cache.contains_key(&gid) {
            return Ok(());
        };
        let offset = self.offsets[gid as usize];
        cursor.seek(offset as usize + self.local_offset)?;
        let contour_count = cursor.read_i16()?;
        let x_min = cursor.read_i16()?;
        let y_min = cursor.read_i16()?;
        let x_max = cursor.read_i16()?;
        let y_max = cursor.read_i16()?;
        let glyph_header = GlyphHeader {
            contour_count,
            x_min,
            y_min,
            x_max,
            y_max,
        };
        let glyph = if contour_count >= 0 {
            let simple = self.parse_simple(cursor, glyph_header)?;
            Glyph::Simple(simple)
        } else {
            let composite = self.parse_composite(cursor, glyph_header)?;
            Glyph::Composite(composite)
        };
        self.glyph_cache.insert(gid, Arc::new(glyph));
        Ok(())
    }
    fn parse_composite(
        &mut self,
        cursor: &mut Cursor,
        glyph_header: GlyphHeader,
    ) -> Result<CompositeGlyph, Error> {
        let resolved = self
            .parse_components(cursor)?
            .into_iter()
            .map(|c| self.resolve_component(c, cursor))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(CompositeGlyph {
            header: glyph_header,
            components: resolved,
        })
    }
    fn resolve_component(
        &mut self,
        component: UnresolvedComponent,
        cursor: &mut Cursor,
    ) -> Result<Component, Error> {
        if !self.glyph_cache.contains_key(&component.gid) {
            self.parse_glyf_block(component.gid, cursor)?;
        }
        if let Some(item) = self.glyph_cache.get(&component.gid) {
            Ok(component.resolve(item.clone()))
        } else {
            Err(Error::Unknown)
        }
    }
    fn parse_components(&mut self, cursor: &mut Cursor) -> Result<Vec<UnresolvedComponent>, Error> {
        let mut components = Vec::new();
        loop {
            let flags = cursor.read_u16()?;
            let gid = cursor.read_u16()?;
            let (arg1, arg2) = self.parse_args(cursor, flags)?;
            let mut transform = self.parse_transform(cursor, flags)?;
            transform.dx = arg1 as f64;
            transform.dy = arg2 as f64;
            components.push(UnresolvedComponent {
                flags,
                gid,
                transform,
            });
            if flags & ComponentFlags::MORE_COMPONENTS as u16 == 0 {
                break;
            }
        }
        Ok(components)
    }
    fn parse_simple(
        &mut self,
        cursor: &mut Cursor,
        header: GlyphHeader,
    ) -> Result<SimpleGlyph, Error> {
        if header.contour_count == 0 {
            return Ok(SimpleGlyph {
                header,
                contours: Vec::new(),
            });
        }
        let mut end_points = Vec::new();
        for _ in 0..header.contour_count {
            end_points.push(cursor.read_u16()?);
        }
        let instruct_length = cursor.read_u16()?;
        cursor.seek(cursor.position() + instruct_length as usize)?;
        let mut flags = Vec::new();
        let num_points = end_points[header.contour_count as usize - 1] + 1;
        let mut i = 0;
        while i < num_points {
            let raw_flag = cursor.read_u8()?;
            let repeat = if raw_flag & SimpleFlags::REPEAT_FLAG as u8 != 0 {
                cursor.read_u8()?
            } else {
                0
            };
            for _ in 0..=repeat {
                flags.push(raw_flag);
                i += 1;
            }
        }
        let x_coordinates = self.read_deltas(
            cursor,
            &flags,
            &SimpleFlags::X_SHORT_VECTOR,
            &SimpleFlags::X_IS_SAME_OR_POSITIVE_X_SHORT_VECTOR,
        )?;
        let y_coordinates = self.read_deltas(
            cursor,
            &flags,
            &SimpleFlags::Y_SHORT_VECTOR,
            &SimpleFlags::Y_IS_SAME_OR_POSITIVE_Y_SHORT_VECTOR,
        )?;
        let mut start = 0;
        let mut curves = Vec::with_capacity(end_points.len());
        for &end in &end_points {
            let end = end as usize;
            let contour_curves =
                curve_from_coords(start, end, &x_coordinates, &y_coordinates, &flags)?;
            curves.push(contour_curves);
            start = end + 1;
        }
        Ok(SimpleGlyph {
            header,
            contours: curves,
        })
    }
    fn read_deltas(
        &self,
        cursor: &mut Cursor,
        flags: &[u8],
        short_flag: &SimpleFlags,
        same_or_positive_flag: &SimpleFlags,
    ) -> Result<Vec<i16>, Error> {
        let mut coords = Vec::with_capacity(flags.len());
        let mut current = 0i16;
        for flag in flags {
            let delta = if flag & *short_flag as u8 != 0 {
                let v = cursor.read_u8()? as i16;
                if flag & *same_or_positive_flag as u8 != 0 {
                    v
                } else {
                    -v
                }
            } else if flag & *same_or_positive_flag as u8 != 0 {
                0
            } else {
                cursor.read_i16()?
            };
            current += delta;
            coords.push(current);
        }
        Ok(coords)
    }
    pub fn get_glyph(&mut self, gid: GlyphId) -> Option<&Arc<Glyph>> {
        self.glyph_cache.get(&gid)
    }
}
fn curve_from_coords(
    start: usize,
    end: usize,
    x: &[i16],
    y: &[i16],
    flags: &[u8],
) -> Result<Vec<BezierTypes>, Error> {
    if end < start {
        return Ok(Vec::new());
    }
    let mut pts: Vec<(Vec2, bool)> = Vec::new();
    for i in start..=end {
        let on_curve = flags[i] & SimpleFlags::ON_CURVE_POINT as u8 != 0;
        pts.push((
            Vec2 {
                x: x[i] as f64,
                y: y[i] as f64,
            },
            on_curve,
        ));
    }
    if pts.is_empty() {
        return Ok(Vec::new());
    }
    let mut expanded: Vec<(Vec2, bool)> = Vec::new();
    for i in 0..pts.len() {
        let (p0, on0) = pts[i];
        let (p1, on1) = pts[(i + 1) % pts.len()];
        expanded.push((p0, on0));
        if !on0 && !on1 {
            expanded.push((
                Vec2 {
                    x: (p0.x + p1.x) * 0.5,
                    y: (p0.y + p1.y) * 0.5,
                },
                true,
            ));
        }
    }
    if !expanded[0].1 {
        let last = expanded.len() - 1;
        let p0 = expanded[0].0;
        let p1 = expanded[last].0;
        expanded.insert(
            0,
            (
                Vec2 {
                    x: (p0.x + p1.x) * 0.5,
                    y: (p0.y + p1.y) * 0.5,
                },
                true,
            ),
        );
    }
    let mut curves = Vec::new();
    let mut i = 0;
    while i + 1 < expanded.len() {
        let (p0, on0) = expanded[i];
        let (p1, on1) = expanded[i + 1];
        if on0 && on1 {
            curves.push(BezierTypes::Linear(LinearBezier::new(
                p0,
                p1,
                EdgeColor::WHITE,
            )));
            i += 1;
        } else if on0 && !on1 {
            let (p2, on2) = expanded[(i + 2) % expanded.len()];
            if !on2 {
                return Err(Error::Unknown);
            }
            curves.push(BezierTypes::Quadratic(QuadraticBezier::new(
                p0,
                p1,
                p2,
                EdgeColor::WHITE,
            )));
            i += 2;
        } else {
            return Err(Error::Unknown);
        }
    }
    Ok(curves)
}
