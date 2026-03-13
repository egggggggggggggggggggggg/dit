use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    hash::Hash,
};

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
pub trait SetOps {
    //Returns a number indicating the amount of items in both sets that satisfy a given sim()
    //condition
    fn intersect(&self, other: &Self, threshold: f32) -> usize;
    //Retuns a number indicating the amount of unique items of both sets combined.
    fn union(&self, other: &Self) -> usize;
}
pub fn jaccard<T: SetOps>(a: &T, b: &T) -> f32 {
    a.intersect(b, 0.5) as f32 / a.union(b) as f32
}
#[derive(Clone, Debug, Default)]
pub struct TextBuf<A: AsRef<str>> {
    pub buf: Vec<A>,
    //Similarity cache for both union and intersect.
    cache: HashMap<A, f32>,
}
impl<A: AsRef<str>> TextBuf<A> {
    pub fn new(buf: Vec<A>) -> Self {
        Self {
            buf,
            cache: HashMap::new(),
        }
    }
}
impl<A: AsRef<str> + Eq + Hash> SetOps for TextBuf<A> {
    fn union(&self, other: &Self) -> usize {
        let mut set: HashSet<_> = self.buf.iter().collect();
        set.extend(other.buf.iter());
        set.len()
    }
    fn intersect(&self, other: &Self, threshold: f32) -> usize {
        let mut counter = 0;
        for a in &self.buf {
            for b in &other.buf {
                let sim_val = dl_distance(a, b);
                if sim_val >= threshold {
                    counter += 1;
                }
            }
        }
        counter
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
