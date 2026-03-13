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
    ///Creates a font file for th
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
    Otf(OtfFont),
    NotLoaded,
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
bitflags::bitflags! {
    struct FontAttributes: u32 {
        const BOLD = 0x01;

    }
}

pub struct FontLoader {
    //The Font table can contain unloaded fonts. the font loader has to match on that enum variant
    //and properly load it and return it back into
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

#[derive(Clone)]
pub struct FileInfo {
    pub name: String,
    pub tokens: Vec<String>,
    pub stripped: String,
    pub path: PathBuf,
}

type FileName = OsString;

pub struct FileFinder {
    pub file_table: HashMap<FileName, FileInfo>,
    pub flattened_array: Vec<FileName>,
}

impl FileFinder {
    pub fn new() -> Self {
        Self {
            file_table: HashMap::new(),
            flattened_array: Vec::new(),
        }
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
                    entries.push(std::fs::read_dir(&path)?);
                    continue;
                }

                if let Some(stem) = path.file_stem() {
                    let name = stem.to_string_lossy().to_string();

                    let (tokens, stripped) = tokenize_and_strip(&name);

                    discovered.push(FileInfo {
                        name,
                        tokens,
                        stripped,
                        path,
                    });
                }
            }
        }

        Ok(discovered)
    }

    pub fn find_file(&self, file_name: &str) {
        // fuzzy search would go here
    }
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
pub fn normalize(str: &str) {}
use image::Rgb;
use thiserror::Error;

use crate::dsa::cache::LruCache;
#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(#[from] std::num::ParseIntError),
}
struct AtlasEntry {}

pub struct AtlasCache {
    cache: LruCache<char, AtlasEntry>,
    //Changing this later on to be more ergonomic or just hard code the values.
    atlas: Atlas<char, Rgb<u8>, ShelfAllocator>,
}
///Changes to implement: Shift the Atlas struct from being the thing that manages all the Atlas
///related stuff to just holding the TextureAtlas. The AtlasCache is responsible for managing the
///UV cords of the texture. 



///Possible Allocator implementations. 
///Buddy Allocator
///Slab Allocator
///Shelf Allocator - this sucks cuz fragmentation heavy
impl AtlasCache {
    pub fn new(capacity: usize, height: u32, width: u32) -> Self {
        let allocator = ShelfAllocator::new(width, height);
        Self {
            cache: LruCache::with_capacity(capacity),
            atlas: Atlas::new(width, height, allocator, 2, false),
        }
    }
    ///This will only return the UV coords for the key if the texture for the glyph exists in the
    ///Atlas itself. If it doesn't exist there then push the new value to the LRUCache. The
    ///LruCache will return the proper character to evict from it's cache. This doesn't handle
    ///ligatures as that would require abstracing over chars or having some way of hasing that.
    ///For future changes, maybe move over from using chars to codepoints for expandability. 
    pub fn get(&mut self, k: &char) {
        self.cache.
    }

}
