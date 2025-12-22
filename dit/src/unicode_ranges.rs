#[derive(Debug, Clone, Copy)]
pub struct UnicodeRange {
    pub start: u32,
    pub end: u32,
    pub name: &'static str,
}
pub const UNICODE_RANGES_TIER1: &[UnicodeRange] = &[
    UnicodeRange {
        start: 0x0020,
        end: 0x007F,
        name: "Basic Latin",
    },
    UnicodeRange {
        start: 0x0080,
        end: 0x00FF,
        name: "Latin-1 Supplement",
    },
    UnicodeRange {
        start: 0x0100,
        end: 0x024F,
        name: "Latin Extended-A/B",
    },
    UnicodeRange {
        start: 0x0250,
        end: 0x02AF,
        name: "IPA Extensions",
    },
    UnicodeRange {
        start: 0x0370,
        end: 0x03FF,
        name: "Greek and Coptic",
    },
    UnicodeRange {
        start: 0x0400,
        end: 0x04FF,
        name: "Cyrillic",
    },
    UnicodeRange {
        start: 0x2000,
        end: 0x206F,
        name: "General Punctuation",
    },
    UnicodeRange {
        start: 0x2190,
        end: 0x21FF,
        name: "Arrows",
    },
    UnicodeRange {
        start: 0x2500,
        end: 0x257F,
        name: "Box Drawing",
    },
    UnicodeRange {
        start: 0x2580,
        end: 0x259F,
        name: "Block Elements",
    },
    UnicodeRange {
        start: 0x25A0,
        end: 0x25FF,
        name: "Geometric Shapes",
    },
    UnicodeRange {
        start: 0xE000,
        end: 0xF8FF,
        name: "Private Use Area (PUA)",
    },
];
pub const UNICODE_RANGES_TIER2: &[UnicodeRange] = &[
    UnicodeRange {
        start: 0x1F300,
        end: 0x1F5FF,
        name: "Misc Symbols & Pictographs",
    },
    UnicodeRange {
        start: 0x1F600,
        end: 0x1F64F,
        name: "Emoticons",
    },
    UnicodeRange {
        start: 0x1F680,
        end: 0x1F6FF,
        name: "Transport & Map Symbols",
    },
    UnicodeRange {
        start: 0x1F900,
        end: 0x1F9FF,
        name: "Supplemental Symbols and Pictographs",
    },
    UnicodeRange {
        start: 0x2B00,
        end: 0x2BFF,
        name: "Misc Symbols and Arrows",
    },
    UnicodeRange {
        start: 0x1F100,
        end: 0x1F1FF,
        name: "Enclosed Alphanumerics",
    },
    UnicodeRange {
        start: 0x1F300,
        end: 0x1FAFF,
        name: "Emoji Extensions",
    },
];
pub const UNICODE_RANGES_TIER3: &[UnicodeRange] = &[
    UnicodeRange {
        start: 0x3000,
        end: 0x303F,
        name: "CJK Symbols and Punctuation",
    },
    UnicodeRange {
        start: 0x3040,
        end: 0x30FF,
        name: "Hiragana & Katakana",
    },
    UnicodeRange {
        start: 0x4E00,
        end: 0x9FFF,
        name: "CJK Unified Ideographs",
    },
    UnicodeRange {
        start: 0xFF00,
        end: 0xFFEF,
        name: "Halfwidth/Fullwidth Forms",
    },
    UnicodeRange {
        start: 0x1F000,
        end: 0x1F02F,
        name: "Mahjong & Domino Tiles",
    },
    UnicodeRange {
        start: 0x1F700,
        end: 0x1F77F,
        name: "Alchemical Symbols",
    },
];
