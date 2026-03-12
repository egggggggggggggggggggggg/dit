use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
};
pub mod search;
use font_parser::{CellMetrics, GlyphHeader, TtfFont};

#[derive(Clone, Copy, Debug)]
pub enum UnicodeRange {}
///Any font file loading must implement this trait.
///Missing stuff to implement later on:
///- Ligature loading
///- Handlig
trait Font: Sized {
    ///Creates a font file for th
    fn new(path: &str) -> Option<Self>;
    ///Handles loading of a specified UnicodeRange.
    ///This handles both lookup of the gid + assembling the glyphs into full equations to be used
    ///by the texture atlas assembler. This is a function defined in the trait due to how different
    ///font file types can store their cmap tables differently.
    fn load_unicode_range(range: UnicodeRange);
    fn get_glyf_header(&self, gid: u16) -> Option<&GlyphHeader>;
    ///This assumes a monospace font and usage in a monospace setting.
    fn get_cell_metrics(&self, font_size: f32) -> CellMetrics;
}
enum FontFileTypes {
    Ttf(TtfFont),
    Otf(OtfFont),
}
impl Font for FontFileTypes {
    fn new(path: &str) -> Option<Self> {
        let file_path = Path::new(path);
        match file_path.extension().and_then(|ext| ext.to_str()) {
            Some("ttf") => Some(FontFileTypes::Ttf(TtfFont::new(path).unwrap())),
            Some("otf") => Some(FontFileTypes::Otf(OtfFont {})),
            Some(_) => {
                println!("Unsupported font file type, skipping loading");
                return None;
            }
            None => {
                println!(
                    "The requested file could not be found, falling back to default font file"
                );
                return None;
            }
        }
    }

    fn load_unicode_range(range: UnicodeRange) {}
    fn get_glyf_header(&self, gid: u16) -> Option<&GlyphHeader> {
        None
    }
    fn get_cell_metrics(&self, font_size: f32) -> CellMetrics {
        match self {
            Self::Ttf(ttf) => ttf.get_cell_metriscs(font_size),
            _ => panic!(),
        }
    }
}
bitflags::bitflags! {
    struct FontAttributes: u32 {
        const BOLD = 0x01;

    }
}

pub struct FontLoader {
    font_table: HashMap<&'static str, FontFileTypes>,
    current_font: &'static str,
    font_size: f32,
    font_attributes: FontAttributes,
}
impl FontLoader {
    pub fn new() {}
    ///Searches itself for a font file, if loaded it'll be in font table. Otherwise it consults the
    ///non loaded vec and performs a search for the requested font.
    pub fn load_font_file(&mut self) {}
    ///Looks in common places for font files and saves them to a Vec.
    pub fn discover_font_files() -> Result<Vec<PathBuf>, Error> {
        let mut discovered_files = Vec::new();
        #[cfg(target_os = "linux")]
        let directory_path = Path::new("/usr/share/fonts");
        let mut entries_to_search = vec![std::fs::read_dir(directory_path)?];
        while let Some(entry) = entries_to_search.pop() {
            for dir_entry in entry {
                match dir_entry {
                    Ok(f) => {
                        let path = f.path();
                        if path.is_dir() {
                            entries_to_search.push(std::fs::read_dir(path)?);
                        } else {
                            discovered_files.push(path);
                        }
                    }
                    _ => println!("File issue"),
                }
            }
        }
        Ok(discovered_files)
    }
}

struct FileInfo {
    name: String,
    tokens: Vec<String>,
    stripped: String,
    path: PathBuf,
}
//Finds a specific file via fuzzy searching.
//
type FileName = OsString;
pub struct FileFinder {
    ///Contains the actual path info
    pub file_table: HashMap<FileName, FileInfo>,
    ///Flattened stuff
    pub flattened_array: Vec<FileName>,
}
impl FileFinder {
    pub fn new() -> Self {
        Self {
            file_table: HashMap::new(),
            flattened_array: Vec::new(),
        }
    }
    pub fn yank_files(path: &str) -> Result<Vec<String>, Error> {
        let mut discovered_files = Vec::new();
        #[cfg(target_os = "linux")]
        let directory_path = Path::new(path);
        let mut entries_to_search = vec![std::fs::read_dir(directory_path)?];
        while let Some(entry) = entries_to_search.pop() {
            for dir_entry in entry {
                match dir_entry {
                    Ok(f) => {
                        let path = f.path();
                        if path.is_dir() {
                            entries_to_search.push(std::fs::read_dir(path)?);
                        } else {
                            discovered_files.push(strip(&path).unwrap());
                        }
                    }
                    _ => println!("File issue"),
                }
            }
        }
        Ok(discovered_files)
    }
    pub fn find_file(file_name: &str) {}
}
pub fn strip(path: &std::path::PathBuf) -> Option<String> {
    let stem = path.file_stem()?.to_string_lossy();
    let mut out = String::with_capacity(stem.len());
    for c in stem.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        }
    }
    Some(out)
}
pub fn normalize(str: &str) {}
use thiserror::Error;
#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(#[from] std::num::ParseIntError),
}
pub struct OtfFont {}
