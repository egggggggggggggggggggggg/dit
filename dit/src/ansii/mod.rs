use core::panic;
use std::io::Bytes;

use smallvec::SmallVec;

#[allow(non_snake_case)]
//Control bytes for 7 bit mode
const NUL: u8 = 0x00;
const BEL: u8 = 0x07;
const BS: u8 = 0x08;
const LF: u8 = 0x0A;
const CR: u8 = 0x0D;
const ESC: u8 = 0x1B;

// Bytes greater or equal to 0x80  are printable
#[repr(u8)]
pub enum ESC {
    IND = b'D',   // ESC D
    NEL = b'E',   // ESC E
    HTS = b'H',   // ESC H
    RI = b'M',    // ESC M
    SS2 = b'N',   // ESC N
    SS3 = b'O',   // ESC O
    DCS = b'P',   // ESC P
    SPA = b'V',   // ESC V
    EPA = b'W',   // ESC W
    SOS = b'X',   // ESC X
    DECID = b'Z', // ESC Z
    CSI = b'[',   // ESC [
    ST = b'\\',   // ESC \
    OSC = b']',   // ESC ]
    PM = b'^',    // ESC ^
    APC = b'_',   // ESC _
}
#[derive(Debug)]
pub enum State {
    Ground,
    Escape,
    EscapeIntermediate,
    CsiEntry,
    CsiParam,
    CsiIntermediate,
    CsiIgnore,
    DcsEntry,
    DcsParam,
    DcsIntermediate,
    DcsPassthrough,
    DcsIgnore,
    OscString,
}
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
struct Rgb {
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
    fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    fn from_index(n: u8) -> Self {
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

#[derive(Debug, Default, Clone)]
pub struct Cursor {
    pub row: usize,
    pub col: usize,
    pub visible: bool,
    pub blinking: bool,
}

const CSI: &str = "\x1b[";
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
// Upwards and above for this
// #[derive(Debug, Clone, Copy)]
// enum VT220Features {
//             1   132-columns.
//             2   Printer.
//             3   ReGIS graphics.
//             4   Sixel graphics.
//             6   Selective erase.
//             8   User-defined keys.
//             9   National Replacement Character sets.
//         TechnicalCharacters = 15
//             1 6    Locator port.
//             1 7    Terminal state interrogation.
//             1 8    User windows.
//             2 1    Horizontal scrolling.
//             2 2    ANSI color, e.g., VT525.
//             2 8    Rectangular editing.
//             2 9    ANSI text locator (i.e., DEC Locator mode).}
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
enum Intensity {
    #[default]
    Normal,
    Bold,
    Faint,
}
#[derive(Debug, Default, Clone, Copy)]
enum Underline {
    #[default]
    None,
    Single,
    Double,
}
//Maintains the state machine and also calls the respective functions
struct Attributes {
    italic: bool,
    blink: bool,
    inverse: bool,
    hidden: bool,
    strike: bool,
    underline: Underline,
    intensity: Intensity,
    fg: Rgb,
    bg: Rgb,
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
struct Cell {
    ch: char,
    attr: Attributes,
}
struct Screen {
    grid: Vec<Vec<Cell>>,
    cursor: Cursor,
    // Holds the state of attributes for a new character that might be typed in
    char_attributes: Attributes,
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

// https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Functions-using-CSI-_-ordered-by-the-final-character_s_
// reference for implementing the escape codes and function dispatching
impl Screen {}
// THe handler shouldnt panic on unreognized escape codes. rather it should print it out instead

impl Handler for Screen {
    fn bell() {}
    fn csi() {}
    // Possible change to this can be matching on tuples instead of nested match statements
    // Will benchmark it later when I have both versions down
    // Low priority in terms of optimizations for terminal emulator
    fn handle_esc(
        &mut self,
        params: SmallVec<[u16; 8]>,
        intermediate: SmallVec<[u8; 4]>,
        final_byte: u8,
    ) {
        match final_byte {
            b'A' => self.cursor_up(params[0]),
            b'B' => self.cursor_down(params[0]),
            b'C' => self.cursor_left(params[0]),
            b'D' => self.cursor_right(params[0]),
            b'E' => self.next_line(),
            b'F' => self.previous_line(),
            b'G' => self.previous_line(),
            b'H' => self.cursor_position(params[0], params[1]),
            // a look up table might be more suited for this repetitive logic, but it might overcomplicate it aswell.
            // for the sequences that have intermediates always match the intermediate first before interpreting the params;
            b'^' => {}
            b'`' => {}
            b'a' => {}
            b'b' => {}
            b'c' => {}
            b'd' => {}
            b'e' => {}
            b'f' => {}
            b'g' => {}
            b'h' => {}
            b'i' => {}
            b'l' => {}

            b'm' => {
                //Interpret the parmss
                if params.len() == 0 {
                    panic!("Params were not passed in, cannot interpret char attributes")
                }
                // Since this function is being moved to the Parser the char attribute function
                // will be used to update state considering it makes it easier of a seperation
                self.char_attributes(params);
            }
            b'n' => {
                if let Some(inter) = intermediate.get(0) {
                    match inter {
                        b'>' => {}
                        b'?' => {}
                        _ => {}
                    }
                } else {
                    self.device_status_report(params[0]);
                }
            }
            b'p' => {
                if let Some(inter) = intermediate.get(0) {
                    match inter {
                        b'#' => {}
                        _ => {}
                    }
                } else {
                }
            }
            b'q' => {}
            b'r' => {}
            b's' => {}
            b't' => {}
            b'u' => {}
            b'v' => {}
            b'w' => {}
            b'x' => {}
            b'y' => {}
            b'z' => {}
            b'|' => {}
            b'}' => {}
            b'{' => {}
            b'~' => match intermediate[0] {
                _ => println!("?"),
            },
            _ => todo!("Implement the rest of the stuff"),
        }
    }
    #[inline(always)]
    fn cursor_up(&mut self, n: u16) {}
    #[inline(always)]
    fn cursor_down(&mut self, n: u16) {}
    #[inline(always)]
    fn cursor_left(&mut self, n: u16) {}
    #[inline(always)]
    fn cursor_right(&mut self, n: u16) {}
    #[inline(always)]
    fn cursor_position(&mut self, new_x: u16, new_y: u16) {}
    #[inline(always)]
    fn next_line(&mut self) {}
    #[inline(always)]
    fn previous_line(&mut self) {}
    #[inline(always)]
    fn char_attributes(&mut self, params: SmallVec<[u16; 8]>) {
        match params[0] {
            0 => {
                self.char_attributes.reset();
            }
            1 => self.char_attributes.intensity = Intensity::Bold,
            2 => self.char_attributes.intensity = Intensity::Faint,
            3 => self.char_attributes.italic = true,
            4 => self.char_attributes.underline = Underline::Single,
            5 => self.char_attributes.blink = true,
            7 => self.char_attributes.inverse = true,
            8 => self.char_attributes.hidden = true,
            9 => self.char_attributes.strike = true,
            21 => self.char_attributes.underline = Underline::Double,
            22 => self.char_attributes.intensity = Intensity::Normal,
            23 => self.char_attributes.italic = false,
            24 => self.char_attributes.underline = Underline::None,
            25 => self.char_attributes.blink = false,
            27 => self.char_attributes.inverse = false,
            28 => self.char_attributes.hidden = false,
            29 => self.char_attributes.strike = false,
            // Will include 3 bit color later, too tedious to write here
            38 => {
                self.char_attributes.fg = match params[1] {
                    // Third param is useless; specifies color space
                    2 => Rgb::new(params[3] as u8, params[4] as u8, params[5] as u8),
                    5 => Rgb::from_index(params[2] as u8),
                    _ => panic!("Improper param"),
                }
            }
            48 => {
                self.char_attributes.bg = match params[1] {
                    // Third param is useless; specifies color space
                    2 => Rgb::new(params[3] as u8, params[4] as u8, params[5] as u8),
                    5 => Rgb::from_index(params[2] as u8),
                    _ => panic!("Improper param"),
                }
            }
            _ => panic!("Unrecognized initial param in the sequence"),
        }
    }
    fn osc(u: u8) {}
    fn device_status_report(&mut self, param: u16) {}
}
// The handler trait is responsible for the actions that need to be performed
// Said actions include sending output the pty/tty
// Thus the struct in which Handler gets implemented for must have access to a Fd

pub trait Handler {
    fn cursor_up(&mut self, n: u16);
    fn cursor_down(&mut self, n: u16);
    fn cursor_right(&mut self, n: u16);
    fn cursor_left(&mut self, n: u16);
    fn cursor_position(&mut self, new_x: u16, new_y: u16);
    fn next_line(&mut self);
    fn previous_line(&mut self);
    fn char_attributes(&mut self, params: SmallVec<[u16; 8]>);
    fn device_status_report(&mut self, param: u16);
    fn csi();
    fn bell();
    fn osc(u: u8);

    fn handle_esc(
        &mut self,
        params: SmallVec<[u16; 8]>,
        intermediate: SmallVec<[u8; 4]>,
        final_byte: u8,
    );
}
pub struct Parser {
    pub state: State,
    pub params: SmallVec<[u16; 8]>,
    pub intermediates: SmallVec<[u8; 4]>,
    current_param: u16,
}
impl Parser {
    pub fn new() -> Self {
        Self {
            state: State::Ground,
            params: SmallVec::new(),
            intermediates: SmallVec::new(),
            current_param: 0,
        }
    }
    pub fn execute_c0(&mut self, byte: u8) {}
    pub fn consume(&mut self, byte: u8) {
        //C0 bytes take priority so these are handled first

        //anywhere state
        match byte {
            0x18 | 0x1a => self.state = State::Ground,
            0x1b => self.state = State::Escape,
            0x90 => self.state = State::DcsEntry,
            0x9d => self.state = State::OscString,
            0x9b => self.state = State::CsiEntry,
            _ => (),
        }
        match self.state {
            //Reason why despite some of them are within the same range and same state change but different arms
            //is cuz different actions
            State::Ground => match byte {
                0x20..=0x7f => print!("{}", byte as char),
                _ => return,
            },
            State::Escape => match byte {
                0x5b => self.state = State::CsiEntry,
                0x5d => self.state = State::OscString,
                0x50 => self.state = State::DcsEntry,
                0x20..=0x2f => self.state = State::EscapeIntermediate,

                _ => {
                    println!("Unrecognized escape code, defaulting to printing");
                }
            },
            State::EscapeIntermediate => match byte {
                0x30..=0x7e => self.state = State::Ground,
                _ => {
                    panic!()
                }
            },
            State::OscString => match byte {
                0x9c => self.state = State::Ground,
                _ => panic!(),
            },
            State::CsiEntry => match byte {
                0x40..=0x7e => self.state = State::Ground,
                0x20..=0x2f => self.state = State::CsiIntermediate,
                0x3a => self.state = State::CsiIgnore,
                0x30..=0x39 => {
                    self.state = State::CsiParam;
                    // Collects the first byte
                    self.current_param = byte as u16;
                }

                _ => panic!(),
            },
            State::CsiParam => match byte {
                0x40..=0x7e => self.state = State::Ground,
                0x20..=0x2f => self.state = State::CsiIntermediate,
                0x3a => self.state = State::CsiIgnore,
                0x3c..=0x3f => self.state = State::CsiIgnore,
                0x30..=0x39 => self.collect_param(byte),
                _ => panic!(),
            },
            State::CsiIntermediate => match byte {
                0x30..=0x3f => self.state = State::CsiIgnore,
                _ => self.state = State::Ground,
            },
            State::CsiIgnore => match byte {
                0x40..=0x7e => self.state = State::Ground,
                _ => panic!(),
            },
            State::DcsEntry => match byte {
                0x20..=0x2f => self.state = State::DcsIntermediate,
                0x40..=0x7e => self.state = State::DcsPassthrough,
                0x3a => self.state = State::DcsIgnore,
                0x30..=0x39 => self.state = State::DcsParam,
                0x3b => self.state = State::DcsParam,
                0x3c..=0x3f => self.state = State::DcsParam,
                _ => panic!(),
            },
            State::DcsIgnore => match byte {
                0x9c => self.state = State::Ground,
                _ => panic!(),
            },
            State::DcsIntermediate => match byte {
                0x30..=0x3f => self.state = State::DcsIgnore,
                0x40..=0x7e => self.state = State::DcsPassthrough,
                _ => panic!(),
            },
            State::DcsParam => match byte {
                0x3a => self.state = State::DcsIgnore,
                0x3c..=0x3f => self.state = State::DcsIgnore,
                0x40..=0x7e => self.state = State::DcsPassthrough,
                0x20..=0x2f => self.state = State::DcsIntermediate,
                _ => panic!(),
            },
            State::DcsPassthrough => match byte {
                0x9c => self.state = State::Ground,
                _ => panic!(),
            },
        }
        println!("{:?}", self.state);
    }
    #[inline(always)]
    pub fn collect_param(&mut self, byte: u8) {
        self.current_param = self.current_param * 10 + (byte - b'0') as u16;
    }
}
