use crate::err::ParseError;
use crate::glyph::parse_glyf_block;
use crate::read_utils::{Cursor, f2_14};
use crate::unicode_ranges::{UNICODE_RANGES_TIER1, UnicodeRange};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::result;
use std::sync::Arc;
const MAGIC_NUMBER: u32 = 0x5F0F3CF5;

#[derive(Debug)]
struct TableRecord {
    pub checksum: u32,
    pub table_offset: usize,
    pub length: usize,
}
#[derive(Debug)]
struct Tag([u8; 4]);
impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            std::str::from_utf8(&self.0).unwrap_or("????").to_string()
        )
    }
}
pub struct TtfFont<'a> {
    data: Cursor<'a>,
    tables: HashMap<[u8; 4], TableRecord>,
    offsets: Option<Vec<u32>>,
    maxp: Maxp,
    glyf_cache: HashMap<u32, Glyph>,
}
impl<'a> TtfFont<'a> {}
enum Glyph {
    Simple(Arc<Simple>),
    Composite(Arc<Composite>),
}
struct GlyphHeader {}
struct Simple {
    glyph_header: GlyphHeader,
}
struct Composite {}

//given a list of tables parse them
pub fn parse_tables() {}

pub fn entry() {}
pub fn parse_command() {}

pub fn parse_file(path: &str) -> Result<(), ParseError> {
    let mut cursor = match read_file(path) {
        Ok(curs) => curs,
        Err(_e) => panic!(
            "Failed to acquire a cursor for the file specified : {}",
            path
        ),
    };
    let table = match parse_header(&mut cursor) {
        Ok(table) => table,
        Err(_e) => panic!("Failed to decipher the offset table"),
    };
    formatted_print(&table);
    let head_info = table
        .get(b"head")
        .ok_or(ParseError::MissingRequiredTable("head"))?;
    let maxp_info = table
        .get(b"maxp")
        .ok_or(ParseError::MissingRequiredTable("maxp"))?;
    let loca_info = table
        .get(b"loca")
        .ok_or(ParseError::MissingRequiredTable("loca"))?;
    let hhea_info = table
        .get(b"hhea")
        .ok_or(ParseError::MissingRequiredTable("hhea"))?;
    let hmtx_info = table
        .get(b"hmtx")
        .ok_or(ParseError::MissingRequiredTable("hmtx"))?;
    let cmap_info = table
        .get(b"cmap")
        .ok_or(ParseError::MissingRequiredTable("cmap"))?;
    let glyf_info = table
        .get(b"glyf")
        .ok_or(ParseError::MissingRequiredTable("glyf"))?;
    let head = parse_head(&mut cursor, &head_info)?;
    let maxp = parse_maxp(&mut cursor, &maxp_info)?;
    let offsets = parse_loca(&mut cursor, &loca_info, head.index_to_loc_format)?;
    if offsets.len() != maxp.glyph_count as usize + 1 {
        println!("count: {}, length: {}", maxp.glyph_count, offsets.len());
        panic!("Offsets were invalid");
    }
    let hhea = parse_hhea(&mut cursor, &hhea_info)?;
    let hmtx = parse_hmtx(&mut cursor, &hmtx_info)?;
    let cmap = parse_cmap(&mut cursor, cmap_info)?;
    let glyf_cache: HashMap<u32, u32> = HashMap::new();
    let _ = parse_glyf(
        &mut cursor,
        &glyf_info,
        &UNICODE_RANGES_TIER1[0],
        &offsets,
        &cmap,
    );
    return Ok(());
}

//parses for a specific unicode range
//
fn parse_range() {
    //get back the gid for the respective groups asked for
}

fn parse_glyf(
    cursor: &mut Cursor,
    info: &TableInfo,
    range: &UnicodeRange,
    offsets: &Vec<u32>,
    cmap: &Vec<CMapGroup>,
) -> Result<(), ParseError> {
    let mut map = HashMap::new();
    for c in range.start..range.end {
        let gid = lookup(cmap, c).unwrap();
        let start = offsets[gid as usize];
        let end = offsets[gid as usize + 1 as usize];
        let length = end - start;
        if length == 0 {
            continue;
        }
        map.insert(c, gid);
        //just throw in the cursor and cache along with the look up function
        //rewrite this to have more info for parsing
    }
    Ok(())
}

struct CMapSubtable {
    platform_id: u16,
    encoding_id: u16,
    offset: usize,
    format: u16,
}
fn select_best_cmap(subtables: &[CMapSubtable]) -> Option<&CMapSubtable> {
    subtables
        .iter()
        .find(|s| s.platform_id == 0 && s.encoding_id == 4 && s.format == 12)
        .or_else(|| {
            subtables
                .iter()
                .find(|s| s.platform_id == 3 && s.encoding_id == 10 && s.format == 12)
        })
        .or_else(|| {
            subtables
                .iter()
                .find(|s| s.platform_id == 0 && s.encoding_id == 3 && s.format == 4)
        })
        .or_else(|| {
            subtables
                .iter()
                .find(|s| s.platform_id == 3 && s.encoding_id == 1 && s.format == 4)
        })
}
fn lookup(groups: &[CMapGroup], c: u32) -> Option<u32> {
    for g in groups {
        if g.start_char <= c && c <= g.end_char {
            return Some(g.start_glyph + (c - g.start_char));
        }
    }
    None
}
fn parse_cmap(cursor: &mut Cursor, info: &TableInfo) -> Result<Vec<CMapGroup>, ParseError> {
    cursor.seek(info.table_offset)?;
    println!("{}", info.length);
    let version = cursor.read_u16()?;
    let num_tables = cursor.read_u16()?;
    let mut subtables = Vec::new();
    for _ in 0..num_tables {
        let platform_id = cursor.read_u16()?;
        let encoding_id = cursor.read_u16()?;
        let offset = cursor.read_u32()? as usize;
        let saved = cursor.position();
        cursor.seek(info.table_offset + offset)?;
        let format = cursor.read_u16()?;
        cursor.seek(saved)?;
        subtables.push(CMapSubtable {
            platform_id,
            encoding_id,
            offset,
            format,
        });
    }
    let chosen = select_best_cmap(&subtables).ok_or(ParseError::InvalidTag)?;
    cursor.seek(info.table_offset + chosen.offset)?;
    match chosen.format {
        4 => parse_format4(cursor),
        12 => parse_format12(cursor),
        _ => return Err(ParseError::InvalidTag),
    }
}
#[derive(Debug)]
struct CMapGroup {
    start_char: u32,
    end_char: u32,
    start_glyph: u32,
}
fn parse_format4(cursor: &mut Cursor) -> Result<Vec<CMapGroup>, ParseError> {
    let format = cursor.read_u16()?;
    let length = cursor.read_u16()?;
    let language = cursor.read_u16()?;
    let segCountX2 = cursor.read_u16()?;
    let search_range = cursor.read_u16()?;
    let entry_selector = cursor.read_u16()?;
    let range_shift = cursor.read_u16()?;
    let mut end_codes = Vec::new();
    let mut start_codes = Vec::new();
    let mut id_deltas = Vec::new();
    let mut id_range_offset = Vec::new();
    let mut glyph_id_array = Vec::new();
    let seg_count = segCountX2 as usize / 2 as usize;
    for _ in 0..seg_count {
        end_codes.push(cursor.read_u16()?);
    }
    let reserved_pad = cursor.read_u16()?;
    for _ in 0..seg_count {
        start_codes.push(cursor.read_u16()?);
    }
    for _ in 0..seg_count {
        id_deltas.push(cursor.read_i16()?);
    }
    for _ in 0..seg_count {
        id_range_offset.push(cursor.read_u16()?);
    }
    let bytes_read = 16 + seg_count * 8;
    let remaining = length as usize - bytes_read;
    for _ in 0..(remaining / 2) {
        glyph_id_array.push(cursor.read_u16()?);
    }
    let mut groups = vec![];
    for i in 0..seg_count {
        let start = start_codes[i];
        let end = end_codes[i];
        if start == 0xFFFF && end == 0xFFFF {
            continue;
        }
        let delta = id_deltas[i] as i32;
        let range_offset = id_range_offset[i];
        let mut c = start as u32;
        while c <= end as u32 {
            let glyph = if range_offset == 0 {
                ((c as i32 + delta) & 0xFFFF) as u16
            } else {
                let roffset = (range_offset / 2) as usize;
                let idx = roffset + (c as usize - start as usize) + i - seg_count;
                if idx >= glyph_id_array.len() {
                    c += 1;
                    continue;
                }
                let raw = glyph_id_array[idx];
                if raw == 0 {
                    c += 1;
                    continue;
                }
                ((raw as i32 + delta) & 0xFFFF) as u16
            };
            if glyph == 0 {
                c += 1;
                continue;
            }
            let run_start_c = c;
            let run_start_g = glyph as u32;
            c += 1;
            while c <= end as u32 {
                let next = if range_offset == 0 {
                    ((c as i32 + delta) & 0xFFFF) as u16
                } else {
                    let roffset = (range_offset / 2) as usize;
                    let idx = roffset + (c as usize - start as usize) + i - seg_count;
                    if idx >= glyph_id_array.len() {
                        break;
                    }
                    let raw = glyph_id_array[idx];
                    if raw == 0 {
                        break;
                    }
                    ((raw as i32 + delta) & 0xFFFF) as u16
                };
                if next as u32 != run_start_g + (c - run_start_c) {
                    break;
                }
                c += 1;
            }
            groups.push(CMapGroup {
                start_char: run_start_c,
                end_char: c - 1,
                start_glyph: run_start_g,
            });
        }
    }
    Ok(groups)
}
fn parse_format12(cursor: &mut Cursor) -> Result<Vec<CMapGroup>, ParseError> {
    let format = cursor.read_u16()?;
    if format != 12 {
        return Err(ParseError::InvalidTag);
    }
    let _reserved = cursor.read_u16()?;
    let _length = cursor.read_u32()?;
    let _language = cursor.read_u32()?;
    let num_groups = cursor.read_u32()?;
    let mut groups = Vec::with_capacity(num_groups as usize);
    for _ in 0..num_groups {
        groups.push(CMapGroup {
            start_char: cursor.read_u32()?,
            end_char: cursor.read_u32()?,
            start_glyph: cursor.read_u32()?,
        });
    }
    Ok(groups)
}
#[derive(Debug)]
struct LongHorMetric {
    advance_width: u16,
    lsb: i16,
}
fn parse_hmtx(cursor: &mut Cursor, info: &TableInfo) -> Result<(), ParseError> {
    match cursor.seek(info.table_offset) {
        Ok(()) => {}
        Err(_e) => panic!("Whoops! Out of bounds"),
    }
    println!("{}", info.length);
    Ok(())
}
#[derive(Debug)]
struct Hhea {
    major: u16,
    minor: u16,
    ascender: i16,
    descender: i16,
    line_gap: i16,
    advance_width_max: u16,
    min_left_bearing: i16,
    min_right_bearing: i16,
    x_max_extent: i16,
    caret_slope_rise: i16,
    caret_slope_run: i16,
    caret_offset: i16,
    metric_data_format: i16,
    hmetric_count: u16,
}
fn parse_hhea(cursor: &mut Cursor, info: &TableInfo) -> Result<Hhea, ParseError> {
    match cursor.seek(info.table_offset) {
        Ok(()) => {}
        Err(_e) => panic!("Whoops! Out of bounds"),
    }
    let major = cursor.read_u16()?;
    let minor = cursor.read_u16()?;
    let ascender = cursor.read_i16()?;
    let descender = cursor.read_i16()?;
    let line_gap = cursor.read_i16()?;
    let advance_width_max = cursor.read_u16()?;
    let min_left_bearing = cursor.read_i16()?;
    let min_right_bearing = cursor.read_i16()?;
    let x_max_extent = cursor.read_i16()?;
    let caret_slope_rise = cursor.read_i16()?;
    let caret_slope_run = cursor.read_i16()?;
    let caret_offset = cursor.read_i16()?;
    let _reserved = cursor.read_i64()?;
    let metric_data_format = cursor.read_i16()?;
    let hmetric_count = cursor.read_u16()?;
    Ok(Hhea {
        major,
        minor,
        ascender,
        descender,
        line_gap,
        advance_width_max,
        min_left_bearing,
        min_right_bearing,
        x_max_extent,
        caret_slope_rise,
        caret_slope_run,
        caret_offset,
        metric_data_format,
        hmetric_count,
    })
}

pub fn formatted_print(table: &HashMap<[u8; 4], TableInfo>) {
    for (tag, value) in table {
        let res = std::str::from_utf8(tag).unwrap_or("????").to_string();
        println!("{:<4} -> {:?}", res, value);
    }
}

#[derive(Debug)]
pub struct TableInfo {
    pub checksum: u32,
    pub table_offset: usize,
    pub length: usize,
}
#[derive(Debug)]
pub struct Head {
    pub major: u16,
    pub minor: u16,
    pub font_revision: f32,
    pub checksum: u32,
    pub magic_number: u32,
    pub flags: u16,
    pub units_per_em: u16,
    pub created: i64,
    pub modified: i64,
    pub x_min: i16,
    pub y_min: i16,
    pub x_max: i16,
    pub y_max: i16,
    pub mac_style: u16,
    pub lowest_rec_ppem: u16,
    pub font_direction_hint: i16,
    pub index_to_loc_format: i16,
    pub glyph_data_format: i16,
}
pub fn parse_head(cursor: &mut Cursor, info: &TableInfo) -> Result<Head, ParseError> {
    match cursor.seek(info.table_offset) {
        Ok(()) => {}
        Err(_e) => panic!("Out of bounds read"),
    }
    let major = cursor.read_u16()?;
    let minor = cursor.read_u16()?;
    let font_rev_major = cursor.read_u16()? as u32;
    let font_rev_minor = cursor.read_u16()? as u32;
    let font_revision = (font_rev_minor << 16 | font_rev_minor) as f32 / (1 << 16) as f32;
    let checksum = cursor.read_u32()?;
    let magic_number = cursor.read_u32()?;
    if magic_number != MAGIC_NUMBER {
        return Err(ParseError::InvalidHeader);
    }
    let flags = cursor.read_u16()?;
    let units_per_em = cursor.read_u16()?;
    let created = cursor.read_i64()?;
    let modified = cursor.read_i64()?;
    let x_min = cursor.read_i16()?;
    let y_min = cursor.read_i16()?;
    let x_max = cursor.read_i16()?;
    let y_max = cursor.read_i16()?;
    let mac_style = cursor.read_u16()?;
    let lowest_rec_ppem = cursor.read_u16()?;
    let font_direction_hint = cursor.read_i16()?;
    let index_to_loc_format = cursor.read_i16()?;
    let glyph_data_format = cursor.read_i16()?;
    Ok(Head {
        major,
        minor,
        font_revision,
        checksum,
        magic_number,
        flags,
        units_per_em,
        created,
        modified,
        x_min,
        y_min,
        x_max,
        y_max,
        mac_style,
        lowest_rec_ppem,
        font_direction_hint,
        index_to_loc_format,
        glyph_data_format,
    })
}
#[derive(Debug)]
struct Maxp {
    vers_major: u32,
    vers_minor: u32,
    glyph_count: u16,
    max_points: u16,
    max_contours: u16,
    max_composite_points: u16,
    max_composite_contours: u16,
    max_zones: u16,
    max_twilight_points: u16,
    max_storage: u16,
    max_function_defs: u16,
    max_stack_elements: u16,
    max_size_of_instructions: u16,
    max_component_elements: u16,
    max_component_depth: u16,
}

fn parse_maxp(cursor: &mut Cursor, info: &TableInfo) -> Result<Maxp, ParseError> {
    match cursor.seek(info.table_offset) {
        Ok(()) => {}
        Err(_e) => panic!("out of bounds"),
    };
    let vers_major = cursor.read_u16()? as u32;
    let vers_minor = cursor.read_u16()? as u32;
    let glyph_count = cursor.read_u16()?;
    let max_points = cursor.read_u16()?;
    let max_contours = cursor.read_u16()?;
    let max_composite_points = cursor.read_u16()?;
    let max_composite_contoursim = cursor.read_u16()?;
    let max_zones = cursor.read_u16()?;
    let max_twilight_points = cursor.read_u16()?;
    let max_storage = cursor.read_u16()?;
    let max_function_defs = cursor.read_u16()?;
    let max_stack_elements = cursor.read_u16()?;
    let max_size_of_instructions = cursor.read_u16()?;
    let max_component_elements = cursor.read_u16()?;
    let max_component_depth = cursor.read_u16()?;
    Ok(Maxp {
        vers_major,
        vers_minor,
        glyph_count,
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
fn parse_loca(cursor: &mut Cursor, info: &TableInfo, format: i16) -> Result<Vec<u32>, ParseError> {
    match cursor.seek(info.table_offset) {
        Ok(()) => {}
        Err(_e) => panic!("Invalid index"),
    }
    let mut offsets: Vec<u32> = Vec::new();
    match format {
        0 => {
            let count = info.length / 2;
            for _ in 0..count {
                let raw = cursor.read_u16()?;
                offsets.push((raw as u32) * 2);
            }
        }
        1 => {
            let count = info.length / 4;
            for _ in 0..count {
                let raw = cursor.read_u32()?;
                offsets.push(raw);
            }
        }
        _ => return Err(ParseError::InvalidTag),
    }
    Ok(offsets)
}
fn parse_header(cursor: &mut Cursor) -> Result<HashMap<[u8; 4], TableInfo>, ParseError> {
    let _sfnt_version = cursor.read_u32()?;
    let num_tables = cursor.read_u16()?;
    let _search_range = cursor.read_u16()?;
    let _entry_selector = cursor.read_u16()?;
    let _range_shift = cursor.read_u16()?;
    let mut table_map = HashMap::new();
    for _ in 0..num_tables {
        let tag: [u8; 4] = cursor.read_u32()?.to_be_bytes();
        let checksum = cursor.read_u32()?;
        let table_offset = cursor.read_u32()? as usize;
        let length = cursor.read_u32()? as usize;
        let table_info = TableInfo {
            checksum,
            table_offset,
            length,
        };

        table_map.insert(tag, table_info);
    }
    Ok(table_map)
}

fn read_file<'a>(path: &'a str) -> std::io::Result<Cursor> {
    let mut data = Vec::new();
    File::open(path)?.read_to_end(&mut data)?;
    Ok(Cursor::new(&data))
}
