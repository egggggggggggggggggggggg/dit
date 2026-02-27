use smallvec::SmallVec;
use std::collections::VecDeque;
use std::io::Bytes;
use std::os::linux::raw::stat;
use std::panic;

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
#[derive(Clone, Debug)]
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

#[derive(Default, Clone)]
struct Cell {
    ch: char,
    attr: Attributes,
}

struct Row {
    data: Vec<Cell>,
}
impl Row {
    fn new(cell_count: usize) -> Self {
        Self {
            data: vec![Cell::default(); cell_count],
        }
    }
}
#[derive(Default)]
enum ScreenMode {
    // Replace the current char
    Insert,
    // Inserts a new character
    // between the current two around the cursor
    #[default]
    OnCursor,
}
// This assumes any application using the term emu
// has access to only the current rows visisble to the screen
struct Screen {
    // Both handle the buffers that don't get shown to the user
    // Holds the rows that are hidden when the user scrolls down
    look_back_buffer: Vec<Vec<Cell>>,
    // HOlds the rows that are hidden when the user scrolls up
    look_ahead_buffer: Vec<Vec<Cell>>,

    // This acts as a visual buffer aswell
    // Contains both input and output buffer
    grid: VecDeque<Vec<Cell>>,
    col_size: usize,
    row_size: usize,
    cursor: Cursor,
    // Holds the state of attributes for a new character that might be typed in
    char_attributes: Attributes,
    utf8_decoder: Utf8Decoderr,
    // Thing that gets displayed on the terminal
    input_buffer: Vec<&'static str>,
    mode: ScreenMode,
}

//Methods for scrolling rows :
// Mouse scrolling
// Arrow keys for going down and up
// For the alt screen buffer the application utilizing it must
// Implement the controls that send the ANSI escape codes ofr the respective direction
// Of movement
// ex:  neovim utilizes hjkl. internally they map these keys to ansi escape codes they send
// to the term emu

impl Screen {
    fn new(&mut self, col_size: usize, row_size: usize) -> Self {
        Self {
            look_ahead_buffer: Vec::new(),
            look_back_buffer: Vec::new(),
            grid: VecDeque::with_capacity(col_size),
            col_size,
            row_size,
            cursor: Cursor::default(),
            char_attributes: Attributes::default(),
            utf8_decoder: Utf8Decoderr::new(),
            input_buffer: Vec::new(),
            mode: ScreenMode::default(),
        }
    }
    fn write_to_screen(&mut self, char: char) {
        if self.cursor.col + 1 == self.col_size {
            // Must go to the next row
            if self.cursor.row + 2 == self.row_size {
                self.pushback_row();

                // writes to this row
                // now we can safely write to this rows
            }
        }
        let row = &self.grid[self.cursor.row];
        // gets the row
    }
    fn write_at_cursor() {}
    fn get_row(&mut self) {}
    fn alternate_screen_buffer(&mut self) {}
    fn pushback_row(&mut self) {
        // the oldest row so the first in the vec gets
        match self.grid.pop_front() {
            None => panic!("THIS SHOULD NEVER HAPPEN"),
            Some(row) => {
                self.look_back_buffer.push(row);
                self.grid.push_back(vec![]);
            }
        }
    }
    fn change_viewport() {}
    /// Resizes the windows to the given cell * row count
    fn resize(&mut self, new_col: usize, new_row: usize) {
        let prev_cell_count = self.col_size * self.row_size;
        
    }

    fn pushfront_row(&mut self) {}
    fn make_new_row(&mut self) {
        // Pushes the oldest row into the lookback buffer
    }
}

// Text Insertion branch of the system
//Insert mode - Inserts text at position and replaces the previous characters at the insert
//position.
//Default mode - Inserts text at the current position and puts any characters ahead of the cursor
//ahead by the amount of text inserted
//For the default case where text gets pushed it requires that the screen be able to handle
//allocating new space for it.
//Cases:
//End of row:
//Moves the cursor to the next row and inserts the character there
//Final row in the screen - requires a new row to be created and the oldest row(top row) to be
//pushed into the look_back_buffer.
//
// Navigatng the rows
// Use arrows keys or ansi escape codes
// Two buffers + the main buffer. The two buffers are for hidden rows like the ones that are ahead
// when the user scrolls up and the look beehind buffer when the user scrolls down or the capacity
// of the main screen is filled. For the main buffer it utilizes a ring buffer for efficient
// insertions of rows. For when the screen size resizes
//

// https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Functions-using-CSI-_-ordered-by-the-final-character_s_
// reference for implementing the escape codes and function dispatching
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
            b'^' => {
                
            }
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
    fn accumluate_utf8(&mut self, byte: u8) {
        self.utf8_decoder.decode_ascii_stream(&[byte], output);
    }
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

    fn accumluate_utf8(&mut self, byte: u8);
    fn handle_esc(
        &mut self,
        params: SmallVec<[u16; 8]>,
        intermediate: SmallVec<[u8; 4]>,
        final_byte: u8,
    );
}
const REPLACEMENT: char = '\u{FFFD}';
// https://bjoern.hoehrmann.de/utf-8/decoder/dfa/
// Lookup table and algorithm here
// Table driven state machine for utf-8 decoding
static UTF8D: [u8; 400] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9,
    7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
    8, 8, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    0xa, 0x3, 0x3, 0x3, 0x3, 0x3, 0x3, 0x3, 0x3, 0x3, 0x3, 0x3, 0x3, 0x4, 0x3, 0x3, 0xb, 0x6, 0x6,
    0x6, 0x5, 0x8, 0x8, 0x8, 0x8, 0x8, 0x8, 0x8, 0x8, 0x8, 0x8, 0x8, 0x0, 0x1, 0x2, 0x3, 0x5, 0x8,
    0x7, 0x1, 0x1, 0x1, 0x4, 0x6, 0x1, 0x1, 0x1, 0x1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 0, 1, 1, 1, 1, 1, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 3, 1, 3, 1, 1, 1, 1, 1, 1, 1, 3, 1, 1, 1, 1, 1, 3, 1, 3, 1, 1, 1, 1, 1,
    1, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
];

const UTF8_ACCEPT: u32 = 0;
const UTF8_REJECT: u32 = 1;
struct Utf8Decoderr {
    state: u32,
    codep: u32,
}
impl Utf8Decoderr {
    pub fn new() -> Self {
        Self {
            state: UTF8_ACCEPT,
            codep: 0,
        }
    }
    #[inline(always)]
    pub fn decode(&mut self, byte: u8) -> u32 {
        let ty = UTF8D[byte as usize] as u32;
        self.codep = if self.state != 0 {
            (self.codep << 6) | (byte as u32 & 0x3F)
        } else {
            (0xFF >> ty) & byte as u32
        };
        self.state = UTF8D[256 + (self.state as usize) * 16 + ty as usize] as u32;
        self.state
    }
    /// User must provide there own buffer to write the characters into
    // Makes the logic less complicated
    fn decode_ascii_stream(&mut self, input: &[u8], output: &mut Vec<char>) {
        // Persistent thingie
        for &byte in input {
            let s = self.decode(byte);
            if s == UTF8_ACCEPT {
                // Checks if the codep can be converted to a char
                if let Some(ch) = std::char::from_u32(self.codep) {
                    output.push(ch);
                } else {
                    panic!("Invalid codepoint");
                }
            } else if s == UTF8_REJECT {
                output.push('\u{FFFD}');
                self.state = UTF8_ACCEPT; // reset state
            }
        }
    }
}
#[inline(always)]
pub fn decode(state: &mut u32, codep: &mut u32, byte: u8) -> u32 {
    let ty = UTF8D[byte as usize] as u32;
    *codep = if *state != 0 {
        (*codep << 6) | (byte as u32 & 0x3F)
    } else {
        (0xFF >> ty) & byte as u32
    };
    *state = UTF8D[256 + (*state as usize) * 16 + ty as usize] as u32;
    *state
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_ascii_stream() {
        let input = "Hello, 世界!".as_bytes();
        let mut state = UTF8_ACCEPT;
        let mut codep = 0;
        let mut output = Vec::new();

        for &byte in input {
            let s = decode(&mut state, &mut codep, byte);
            if s == UTF8_ACCEPT {
                // Codepoint fully decoded, push it
                if let Some(ch) = std::char::from_u32(codep) {
                    output.push(ch);
                } else {
                    panic!("Invalid codepoint");
                }
            } else if s == UTF8_REJECT {
                output.push('\u{FFFD}');
                state = UTF8_ACCEPT; // reset state
            }
        }

        let result: String = output.iter().collect();
        assert_eq!(result, "Hello, 世界!");
        println!("Decoded output: {}", result);
    }
}
// Need to have a state machine for utf-8 decoding aswell

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
    pub fn consume<H: Handler>(&mut self, byte: u8, handler: &mut H) {
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
                0x20..=0x7f => handler.accumluate_utf8(byte),
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
