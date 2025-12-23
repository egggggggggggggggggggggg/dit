use std::{collections::HashMap, sync::Arc};

use crate::{glyph::parse_simple, parse::{Cursor, GlyphCache, GlyphID, error::Error}};
pub struct SimpleGlyph {
    header: GlyphHeader,
}
pub struct CompositeGlyph {
    components: Vec<Component>,
    header: GlyphHeader,
}

#[derive(Debug, Clone)]
struct GlyphHeader {
    contour_count: i16,
    x_min: i16,
    y_min: i16,
    x_max: i16,
    y_max: i16,
}
pub enum Glyph {
    Simple(SimpleGlyph),
    Composite(CompositeGlyph),
}
//should probably own a slice into the data
struct Glyf<'a> {
    offsets: Vec<u32>,
    glyph_cache: GlyphCache,
    cursor: Cursor<'a>,
}
type TMat = [f32; 4];

pub struct ComponentFlags {}
struct UnresolvedComponent {
    flags: u16,
    gid: GlyphID,
    arg1: i32,
    arg2: i32,
    transform_data: Option<TMat>,
}
impl UnresolvedComponent {
    fn resolve(self, reference: Arc<Glyph>) -> Component {
        Component {
            flags: self.flags,
            gid: self.gid,
            reference: reference,
            arg1: self.arg1,
            arg2: self.arg2,
            transform_data: self.transform_data,
        }
    }
}
pub struct Component {
    pub flags: u16,
    pub gid: u16,
    pub reference: Arc<Glyph>,
    pub arg1: i32,
    pub arg2: i32,
    pub transform_data: Option<TMat>,
}

impl<'a> Glyf<'a> {
    fn parse_composite(&mut self) -> Result<(), Error> {
        let mut parse_stack = vec![]
        let mut unresolved_stack = Vec::new();
        let mut resolved = Vec::new();
        while let Some(gid) = parse_stack.pop() {
            //parse_glyf_block must properly align the cursor prior to calling this function
            //do the composite glyf parsing thing here
            let raw_flags = self.cursor.read_u16()?;
            let gid = self.cursor.read_u16()? as u32;
            let (arg1, arg2) = self.parse_args()?;
            let transform = self.parse_transform();
            //go through the gids and set
            unresolved_stack.push(UnresolvedComponent {
                flags: ComponentFlags {},
                gid,
                arg1,
                arg2,
                transform_data: transform,
            });
        }
        self.resolve_composite(&mut unresolved_stack);
        Ok(())
    }
    fn resolve_composite(&mut self, unresolved_stack: &mut Vec<UnresolvedComponent>) {
        let resolved_components = Vec::new();
        while let Some(unresolved_comp) = unresolved_stack.pop() {
            let gid = unresolved_comp.gid;
            if let Some(component) = self.glyph_cache.get(&gid) {
                unresolved_comp.resolve(Arc::clone(component));
            } else {

                //parse the glyf_block again
            }
        }
    }
    fn parse_glyf_block(&mut self, cursor: &mut Cursor, offset: usize) -> Result<(), Error> {
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
        if contour_count >= 0 {
            self.parse_simple();
        } else {
            self.parse_composite();
        }
        Ok(())
    }
}
impl <'a> Glyf<'a> {
    fn new(data: &'a [u8], offsets: Vec<u32>) -> Self {
        Self {
            offsets,
            glyph_cache: GlyphCache::new(),
            cursor: Cursor::set(data, 0),
        }
    }
    fn parse_transform(&mut self) -> Option<TMat> {
        None
    }
    fn parse_args(&mut self) -> Result<(i32, i32), Error> {
        Ok((-1, -1))
    }

    fn parse_glyf_block(&mut self, gid: GlyphID) -> Result<(), Error> {
        let offset = *self.offsets.get(gid as usize).ok_or(Error::Test)?;
        let cursor = &mut self.cursor;
        cursor.seek(offset as usize)?;
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
        if contour_count >= 0 {
            self.parse_simple();
        } else {
            self.parse_compositee();
        }
        Ok(())
    }

    fn parse_compositee(&mut self) -> Result<(), Error> {
        let components = self.parse_components()?;
        //attempts to resolve the components
        

        Ok(())
    }
    fn parse_components(&mut self) -> Result<Vec<UnresolvedComponent>, Error> {
        let mut components = Vec::new();
        loop {
            let flags = self.cursor.read_u16()?;
            let gid= self.cursor.read_u16()? as u32;
            let (arg1, arg2) = self.parse_args()?;
            let transform_data = self.parse_transform();
            components.push( UnresolvedComponent {
                flags, gid, arg1, arg2, transform_data,
            });
            if flags & 0x0020  != 0 {
                break;
            }
        }
        Ok(components)
    }
    fn parse_simple(&mut self) {

    }


}