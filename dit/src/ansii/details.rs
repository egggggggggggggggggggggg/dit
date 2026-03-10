bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, Default)]
    pub struct AttrFlags: u16 {
        const BOLD          = 1 << 0;
        const FAINT         = 1 << 1;
        const ITALIC        = 1 << 2;
        const UNDERLINE     = 1 << 3;
        const BLINK         = 1 << 4;
        const INVERSE       = 1 << 5;
        const HIDDEN        = 1 << 6;
        const STRIKE        = 1 << 7;
    }
}

#[derive(Debug, Default, Clone)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
const THEME_SPECIFIC: [u8; 16] = [0u8; 16];
const LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];

impl Rgb {
    // constants for the 3 bit colors
    const BLACK: Self = Self { r: 0, g: 0, b: 0 };
    const RED: Self = Self { r: 255, g: 0, b: 0 };
    const GREEN: Self = Self { r: 0, g: 255, b: 0 };
    const YELLOW: Self = Self {
        r: 255,
        g: 255,
        b: 0,
    };
    const BLUE: Self = Self { r: 0, g: 0, b: 255 };
    const MAGENTA: Self = Self {
        r: 255,
        g: 0,
        b: 255,
    };
    const CYAN: Self = Self {
        r: 0,
        g: 255,
        b: 255,
    };
    const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
    };
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn from_index(n: u8) -> Self {
        if n <= 231 && 16 <= n {
            let i = n - 16;
            let r = i / 36;
            let g = (i & 36) / 6;
            let b = i % 6;
            return Self {
                r: LEVELS[r as usize],
                g: LEVELS[g as usize],
                b: LEVELS[b as usize],
            };
        }
        Self { r: 0, g: 0, b: 0 }
    }
}

#[derive(Debug, Clone, Copy)]
enum TerminalType {
    VT100 = 0,
    VT220 = 1,
    VT240 = 2,
    VT330 = 18,
    VT340 = 19,
    VT320 = 24,
    VT382 = 32,
    VT420 = 41,
    VT510 = 61,
    VT520 = 64,
    VT525 = 65,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Intensity {
    #[default]
    Normal,
    Bold,
    Faint,
}
#[derive(Debug, Default, Clone, Copy)]
pub enum Underline {
    #[default]
    None,
    Single,
    Double,
}
//Maintains the state machine and also calls the respective functions
#[derive(Clone, Debug)]
pub struct Attributes {
    pub italic: bool,
    pub blink: bool,
    pub inverse: bool,
    pub hidden: bool,
    pub strike: bool,
    pub underline: Underline,
    pub intensity: Intensity,
    pub fg: Rgb,
    pub bg: Rgb,
}
impl Default for Attributes {
    fn default() -> Self {
        Self {
            italic: false,
            blink: false,
            inverse: false,
            hidden: false,
            strike: false,
            intensity: Intensity::default(),
            underline: Underline::default(),
            bg: Rgb::default(),
            fg: Rgb::default(),
        }
    }
}
impl Attributes {
    fn reset(&mut self) {
        *self = Self::default();
    }
}
#[derive(Debug, Default)]
enum SimpleColors {
    #[default]
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}
#[derive(Debug, Clone, Copy)]
pub enum DeviceStatusReport {
    Ok,
    CursorPosition(u16, u16),
}
#[derive(Debug, Clone, Copy)]
pub enum DeviceAttributes {
    VT100AdvancedVideo, // CSI ? 1 ; 2 c
    VT101NoOptions,     // CSI ? 1 ; 0 c
    VT132,              // CSI ? 4 ; 6 c
    VT102,              // CSI ? 6 c
    VT131,              // CSI ? 7 c
    VT125(u16),         // CSI ? 12 ; Ps c
    VT220(u16),         // CSI ? 62 ; Ps c
    VT320(u16),         // CSI ? 63 ; Ps c
    VT420(u16),         // CSI ? 64 ; Ps c
    VT510(u16),         // CSI ? 65 ; Ps c
}
