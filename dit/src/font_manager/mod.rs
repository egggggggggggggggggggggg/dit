use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
    hash::Hash,
    path::{Path, PathBuf},
};
pub mod search;
use atlas_gen::{allocator::ShelfAllocator, atlas::Atlas};
use font_parser::{CellMetrics, GlyphHeader, TtfFont};

#[derive(Clone, Copy, Debug)]
pub enum UnicodeRange {}
///Any font file loading must implement this trait.
///Missing stuff to implement later on:
///- Ligature loading
///- Handlig
trait Font: Sized {
    ///Creates a font file struct representation.
    fn new(path: &str) -> Option<Self>;
    ///Handles loading of a specified UnicodeRange.
    ///This handles both lookup of the gid + assembling the glyphs into full equations to be used
    ///by the texture atlas assembler. This is a function defined in the trait due to how different
    ///font file types can store their cmap tables differently.
    fn load_unicode_range(&self, range: UnicodeRange);
    fn get_glyf_header(&self, gid: u16) -> Option<&GlyphHeader>;
    ///This assumes a monospace font and usage in a monospace setting.
    fn get_cell_metrics(&self, font_size: f32) -> CellMetrics;
}
enum FontFileTypes {
    Ttf(TtfFont),
    NotLoaded,
}
impl Font for FontFileTypes {
    fn new(path: &str) -> Option<Self> {
        let file_path = Path::new(path);
        match file_path.extension().and_then(|ext| ext.to_str()) {
            Some("ttf") => Some(FontFileTypes::Ttf(TtfFont::new(path).unwrap())),
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
    fn load_unicode_range(&self, range: UnicodeRange) {
        match self {
            Self::Ttf(font) => {}
            _ => {}
        }
    }
    fn get_glyf_header(&self, gid: u16) -> Option<&GlyphHeader> {
        match self {
            Self::Ttf(font) => font.get_glyf_header(gid),
            _ => None,
        }
    }
    fn get_cell_metrics(&self, font_size: f32) -> CellMetrics {
        match self {
            Self::Ttf(ttf) => ttf.get_cell_metriscs(font_size),
            _ => panic!(),
        }
    }
}
#[derive(Clone)]
pub struct FileInfo {
    pub name: String,
    pub tokens: Vec<String>,
    pub path: PathBuf,
}

pub fn yank_files(path: &str) -> Result<Vec<FileInfo>, Error> {
    let mut discovered = Vec::new();
    let mut entries = vec![std::fs::read_dir(Path::new(path))?];

    while let Some(entry) = entries.pop() {
        for dir_entry in entry {
            let f = match dir_entry {
                Ok(f) => f,
                Err(_) => {
                    println!("File issue");
                    continue;
                }
            };

            let path = f.path();

            if path.is_dir() {
                println!("Path for directory: {:?}", path);
                entries.push(std::fs::read_dir(&path)?);
                continue;
            }
            println!("File name: {:?}", path);
            if let Some(stem) = path.file_stem() {
                println!("File stem exists for this");
                let name = stem.to_string_lossy().to_string();

                let tokens = tokenize(&name);
                discovered.push(FileInfo { name, tokens, path });
            }
        }
    }

    Ok(discovered)
}
#[derive(Copy, Clone, Eq, PartialEq)]
enum CharClass {
    Lower,
    Upper,
    Digit,
    Other,
}
fn classify(c: char) -> CharClass {
    if c.is_ascii_lowercase() {
        CharClass::Lower
    } else if c.is_ascii_uppercase() {
        CharClass::Upper
    } else if c.is_ascii_digit() {
        CharClass::Digit
    } else {
        CharClass::Other
    }
}
pub fn tokenize(name: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    let mut prev = CharClass::Other;
    let mut chars = name.chars().peekable();

    while let Some(c) = chars.next() {
        let class = classify(c);

        if class == CharClass::Other {
            if !current.is_empty() {
                tokens.push(current.to_lowercase());
                current.clear();
            }
            prev = CharClass::Other;
            continue;
        }

        let next = chars.peek().copied().map(classify);

        let boundary = match (prev, class) {
            (CharClass::Lower, CharClass::Upper) => true,
            (CharClass::Digit, CharClass::Lower | CharClass::Upper) => true,
            (CharClass::Lower | CharClass::Upper, CharClass::Digit) => true,
            (CharClass::Upper, CharClass::Upper)
                if matches!(next, Some(CharClass::Lower)) && !current.is_empty() =>
            {
                true
            }

            _ => false,
        };

        if boundary {
            tokens.push(current.to_lowercase());
            current.clear();
        }

        current.push(c);
        prev = class;
    }

    if !current.is_empty() {
        tokens.push(current.to_lowercase());
    }

    tokens
}

fn tokenize_and_strip(name: &str) -> (Vec<String>, String) {
    let mut tokens = Vec::new();
    let mut current = String::new();

    let mut prev_lower = false;

    for c in name.chars() {
        if !c.is_ascii_alphanumeric() {
            if !current.is_empty() {
                tokens.push(current.to_lowercase());
                current.clear();
            }
            prev_lower = false;
            continue;
        }

        if c.is_uppercase() && prev_lower {
            tokens.push(current.to_lowercase());
            current.clear();
        }

        current.push(c);
        prev_lower = c.is_lowercase();
    }

    if !current.is_empty() {
        tokens.push(current.to_lowercase());
    }

    // stripped version
    let mut stripped = String::with_capacity(name.len());
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            stripped.push(c.to_ascii_lowercase());
        }
    }

    (tokens, stripped)
}
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(#[from] std::num::ParseIntError),
}
