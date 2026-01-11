//Required Tables
pub mod cmap;
pub mod glyf;
pub mod head;
pub mod hhea;
pub mod hmtx;
pub mod loca;
pub mod maxp;
pub mod name;
pub mod post;

pub use cmap::*;
pub use glyf::*;
pub use head::*;
pub use hhea::*;
pub use hmtx::*;
pub use loca::*;
pub use maxp::*;
//Optional Tables
pub mod gsub;
pub mod kern;

pub(crate) type GlyphId = u16;

#[derive(Debug)]
pub struct TableRecord {
    pub checksum: u32,
    pub table_offset: usize,
    pub length: usize,
}
