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
#[derive(Debug, Default, Clone)]
pub struct Cursor {
    pub row: usize,
    pub col: usize,
    pub visible: bool,
    pub blinking: bool,
}
const CSI: &str = "\x1b[";
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

//Maintains the state machine and also calls the respective functions
struct Attributes {}
struct Cell {
    ch: char,
    attr: Attributes,
}
struct Screen {
    grid: Vec<Vec<Cell>>,
    cursor: Cursor,
}
// https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Functions-using-CSI-_-ordered-by-the-final-character_s_
// reference for implementing the escape codes and function dispatching
impl Screen {}
impl Handler for Screen {
    fn bell() {}
    fn csi() {}
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
            b'm' => {}
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
    fn char_attributes(&mut self) {}
    fn osc(u: u8) {}
}
pub trait Handler {
    fn cursor_up(&mut self, n: u16);
    fn cursor_down(&mut self, n: u16);
    fn cursor_right(&mut self, n: u16);
    fn cursor_left(&mut self, n: u16);
    fn cursor_position(&mut self, new_x: u16, new_y: u16);
    fn next_line(&mut self);
    fn previous_line(&mut self);
    fn char_attributes(&mut self);
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
    pub fn advance(&mut self, byte: u8) {
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
