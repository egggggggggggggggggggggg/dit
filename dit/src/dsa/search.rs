use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use image::{ImageBuffer, Rgb};

use crate::font_manager::{FileInfo, yank_files};

pub fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();
    let s1 = s1.as_bytes();
    let s2 = s2.as_bytes();
    let width = len2 + 1;
    let mut matrix = vec![0usize; (len1 + 1) * (len2 + 1)];
    for i in 0..=len1 {
        matrix[i * width] = i;
    }
    for j in 0..=len2 {
        matrix[j] = j;
    }
    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1[i - 1] == s2[j - 1] { 0 } else { 1 };
            let deletion = matrix[(i - 1) * width + j] + 1;
            let insertion = matrix[i * width + (j - 1)] + 1;
            let substitution = matrix[(i - 1) * width + (j - 1)] + cost;
            matrix[i * width + j] = deletion.min(insertion).min(substitution);
        }
    }
    matrix[len1 * width + len2]
}
pub fn lcs(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();
    let char1: Vec<char> = s1.chars().collect();
    let char2: Vec<char> = s2.chars().collect();
    lcs_rec(&char1, &char2, len1, len2)
}
pub fn lcs_rec(s1: &[char], s2: &[char], m: usize, n: usize) -> usize {
    if m == 0 || n == 0 {
        return 0;
    }
    if s1[m - 1] == s2[n - 1] {
        return 1 + lcs_rec(s1, s2, m - 1, n - 1);
    } else {
        return lcs_rec(s1, s2, m, n - 1).max(lcs_rec(s1, s2, m - 1, n));
    }
}

pub fn lcs_mem(s1: &str, s2: &str) -> usize {
    let a: Vec<char> = s1.chars().collect();
    let b: Vec<char> = s2.chars().collect();

    let m = a.len();
    let n = b.len();

    let mut memo = vec![None; (m + 1) * (n + 1)];

    lcs_rec_mem(&a, &b, m, n, &mut memo, n + 1)
}

fn lcs_rec_mem(
    a: &[char],
    b: &[char],
    m: usize,
    n: usize,
    memo: &mut [Option<usize>],
    width: usize,
) -> usize {
    if m == 0 || n == 0 {
        return 0;
    }

    let idx = m * width + n;

    if let Some(v) = memo[idx] {
        return v;
    }

    let result = if a[m - 1] == b[n - 1] {
        1 + lcs_rec_mem(a, b, m - 1, n - 1, memo, width)
    } else {
        lcs_rec_mem(a, b, m, n - 1, memo, width).max(lcs_rec_mem(a, b, m - 1, n, memo, width))
    };

    memo[idx] = Some(result);
    result
}
#[derive(Clone, Debug, Default)]
pub struct SimilarityCache<A: AsRef<str> + Eq + Hash + Clone> {
    cache: HashMap<(A, A), f32>, // Cache for pairs (A, A)
}

impl<A: AsRef<str> + Eq + Hash + Clone> SimilarityCache<A> {
    pub fn get_or_compute(&mut self, a: &A, b: &A) -> f32 {
        if let Some(sim) = self.cache.get(&(a.clone(), b.clone())) {
            return *sim;
        }
        let sim_val = dl_distance(a, b);
        self.cache.insert((a.clone(), b.clone()), sim_val);
        self.cache.insert((b.clone(), a.clone()), sim_val);
        sim_val
    }
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct TextBuf<A: AsRef<str>> {
    pub buf: Vec<A>,
}

impl<A: AsRef<str> + Eq + Hash + Clone> TextBuf<A> {
    pub fn new(buf: Vec<A>) -> Self {
        Self { buf }
    }
    pub fn jaccard(&mut self, other: &Self, threshold: f32, cache: &mut SimilarityCache<A>) -> f32 {
        let mut intersect_counter = 0;
        let mut union_counter = 0;
        for a in &self.buf {
            for b in &other.buf {
                let sim_val = cache.get_or_compute(&a.clone(), &b.clone());
                if sim_val >= threshold {
                    intersect_counter += 1;
                    union_counter += 1;
                } else {
                    union_counter += 2;
                }
            }
        }
        intersect_counter as f32 / union_counter as f32
    }
}

#[inline(always)]
fn idx(i: usize, j: usize, width: usize) -> usize {
    i * width + j
}
fn dl(i: usize, j: usize, width: usize, s1: &[char], s2: &[char], memo: &mut [i32]) -> i32 {
    if memo[idx(i, j, width)] != -1 {
        return memo[idx(i, j, width)];
    }

    let result = if i == 0 {
        j as i32
    } else if j == 0 {
        i as i32
    } else {
        let cost = if s1[i - 1] == s2[j - 1] { 0 } else { 1 };

        let mut dist = std::cmp::min(
            std::cmp::min(
                dl(i - 1, j, width, s1, s2, memo) + 1,
                dl(i, j - 1, width, s1, s2, memo) + 1,
            ),
            dl(i - 1, j - 1, width, s1, s2, memo) + cost,
        );

        if i > 1 && j > 1 && s1[i - 1] == s2[j - 2] && s1[i - 2] == s2[j - 1] {
            dist = std::cmp::min(dist, dl(i - 2, j - 2, width, s1, s2, memo) + 1);
        }

        dist
    };

    memo[idx(i, j, width)] = result;
    result
}

fn dl_distance<A: AsRef<str>>(s1: &A, s2: &A) -> f32 {
    let s1 = s1.as_ref();
    let s2 = s2.as_ref();
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    let len1 = s1_chars.len();
    let len2 = s2_chars.len();

    let width = len2 + 1;
    let mut memo = vec![-1i32; (len1 + 1) * width];

    let distance = dl(len1, len2, width, &s1_chars, &s2_chars, &mut memo) as f32;

    let max_len = len1.max(len2) as f32;

    if max_len == 0.0 {
        1.0
    } else {
        1.0 - (distance / max_len)
    }
}
///Contains the default font types that should be supported bymost linux distros.
///If it fails it'll just default to one of the fonts shipped with the application.
mod font_defaults {
    pub const DEJAVU: [&str; 2] = ["deja", "vu"];
    pub const LIBERATION: [&str; 1] = ["liberation"];
    pub const FIRA: [&str; 1] = ["fira"];
}

//if we want terminal emulator reloading we must be able to reload anything related to the config
//which includes font types. This is only kept
const SIMILARITY_THRESHOLD: f32 = 0.7;
struct FontLoader {
    current_font: String,
    current_file_info: FileInfo,
    cache: SimilarityCache<String>,
    font_files: Vec<FileInfo>,
}
impl FontLoader {
    pub fn new() {
        //performs a tree construction on lauch
        //Invokes a defaul t font type that it knows exists.
        let font_file_dir = yank_files("/usr/share/fonts").unwrap();
        let default_font_tokens = font_defaults::DEJAVU;
    }
    #[inline(always)]
    pub fn find_file(
        file_name: &[&str; 2],
        file_infos: &Vec<FileInfo>,
        cache: SimilarityCache<String>,
    ) -> Option<String> {
        let mut best_match = "".to_string();
        let mut min_cost = 0.0f32;
        let t1 = TextBuf::new(file_name.to_vec());
        for file_info in file_infos {
            let tokens = file_info.tokens.clone();
            let mut t2 = TextBuf::new(tokens);
            // let res = t1.jaccard(&mut t2, SIMILARITY_THRESHOLD, cache);
            let res = 0.5;
            if res > min_cost {
                best_match = file_info.name.clone();
                min_cost = res;
            }
        }
        Some(best_match)
    }
}
struct AtlasCache {
    dirty_atlas: bool,
    ///This gets sets to true when the texture atlas cannot contain all the current characters on
    ///the screen which might cause repetitive cache eviction thus hurting performance. This
    ///probably will never happen with a 2048 * 2048 sized texture atlas but just in case.
    should_reallocate: bool,
    uv_table: HashMap<char, [(f32, f32); 2]>,
}
impl AtlasCache {
    fn new() -> Self {
        Self {
            dirty_atlas: false,
            should_reallocate: false,
            uv_table: HashMap::new(),
        }
    }
    fn get_char(&mut self, codepoint: u16) {
        //check if we can get them from the UvTable.
    }
}

pub struct AtlasEntry {}

struct TextureAtlas<A> {
    atlas_entries: HashMap<char, AtlasEntry>,
    uv_table: HashMap<char, ([f32; 2], [f32; 2])>,
    height: u32,
    width: u32,
    padding: u32,
    atlas: ImageBuffer<Rgb<u8>, Vec<u8>>,
    allocator: A,
}

impl TextureAtlas {
    pub fn new(width: u32, height: u32, padding: u32) -> Self {
        Self {
            atlas_entries: HashMap::new(),
            uv_table: HashMap::new(),
            height,
            width,
            padding,
            atlas: ImageBuffer::new(width, height),
        }
    }
    fn add_image(&mut self, key: char, src: ImageBuffer<Rgb<u8>, Vec<u8>>) {
        let (w, h) = src.dimensions();
        let p = self.padding;
        let alloc_w = w + 2 * p;
        let alloc_h = h + 2 * p;
        let (x, y) = self
            .allocator
            .allocate(alloc_w, alloc_h)
            .ok_or("Atlas Full")?;
        for sy in 0..h {
            for sx in 0..w {
                let pixel = *src.get_pixel(sx, sy);
                self.atlas.put_pixel(x + p + sx, y + p + sy, pixel);
            }
        }
    }
    fn get_uv(&mut self, key: char) -> ([f32; 2], [f32; 2]) {}
}
