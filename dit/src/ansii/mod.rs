use smallvec::SmallVec;
use std::panic;

pub mod details;
pub mod utf_decoder;

// #[allow(non_snake_case)]
// //Control bytes for 7 bit mode
// const NUL: u8 = 0x00;
// const BEL: u8 = 0x07;
// const BS: u8 = 0x08;
// const LF: u8 = 0x0A;
// const CR: u8 = 0x0D;
// const ESC: u8 = 0x1B;
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
impl State {
    fn allows_execute(&self) -> bool {
        matches!(
            self,
            State::Ground
                | State::Escape
                | State::EscapeIntermediate
                | State::CsiEntry
                | State::CsiIntermediate
                | State::CsiParam
                | State::CsiIgnore
        )
    }
}

#[derive(Debug, Default, Clone)]
pub struct Cursor {
    pub row: usize,
    pub col: usize,
    pub visible: bool,
    pub blinking: bool,
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

// This assumes any application using the term emu
// has access to only the current rows visisble to the screen

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
    fn char_attributes(&mut self, params: &SmallVec<[u16; 8]>);
    fn device_status_report(&mut self, param: u16);
    fn csi();
    fn bell();
    /// Execute takes in a
    fn execute(&mut self, ctl_seq: u8);
    fn handle_osc(&mut self, u: &Vec<u8>);

    fn accumluate_utf8(&mut self, byte: u8);
}
// Need to have a state machine for utf-8 decoding aswell

#[derive(Debug)]
pub struct Parser {
    pub state: State,
    pub params: SmallVec<[u16; 8]>,
    pub intermediates: SmallVec<[u8; 4]>,
    current_param: u16,
    osc_buffer: Vec<u8>,
}
#[inline(always)]
fn is_execute(byte: u8) -> bool {
    matches!(byte, 0x00..=0x17 | 0x19 | 0x1c..=0x1f)
}
#[inline(always)]
fn anywhere_transition(byte: u8) -> Option<State> {
    match byte {
        0x18 | 0x1a => Some(State::Ground),
        0x1b => Some(State::Escape),
        0x90 => Some(State::DcsEntry),
        0x9d => Some(State::OscString),
        0x9b => Some(State::CsiEntry),
        _ => None,
    }
}
impl Parser {
    pub fn new() -> Self {
        Self {
            state: State::Ground,
            params: SmallVec::new(),
            intermediates: SmallVec::new(),
            current_param: 0,
            osc_buffer: Vec::new(),
        }
    }
    fn handle_esc<H: Handler>(&mut self, _final_byte: u8, _handler: &mut H) {}
    fn handle_csi<H: Handler>(&mut self, final_byte: u8, handler: &mut H) {
        match final_byte {
            b'A' => handler.cursor_up(self.params[0]),
            b'B' => handler.cursor_down(self.params[0]),
            b'C' => handler.cursor_left(self.params[0]),
            b'D' => handler.cursor_right(self.params[0]),
            b'E' => handler.next_line(),
            b'F' => handler.previous_line(),
            b'G' => handler.previous_line(),
            b'H' => handler.cursor_position(self.params[0], self.params[1]),
            b'I' => {}
            b'J' => {}
            b'K' => {}
            b'L' => {}
            b'M' => {}
            b'P' => {}
            b'S' => {}
            b'T' => {}
            b'X' => {}
            b'Z' => {}

            // a look up table might be more suited for this repetitive logic, but it might overcomplicate it aswell.
            // for the sequences that have intermediates always match the intermediate first before interpreting the self.params;
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
                if self.params.len() == 0 {
                    panic!("self.params were not passed in, cannot interpret char attributes")
                }
                // Since this function is being moved to the Parser the char attribute function
                // will be used to update state considering it makes it easier of a seperation
                handler.char_attributes(&self.params);
            }
            b'n' => {
                if let Some(inter) = self.intermediates.get(0) {
                    match inter {
                        b'>' => {}
                        b'?' => {}
                        _ => {}
                    }
                } else {
                    handler.device_status_report(self.params[0]);
                }
            }
            b'p' => {
                if let Some(inter) = self.intermediates.get(0) {
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
            b'~' => match self.intermediates[0] {
                _ => println!("?"),
            },
            _ => todo!("Implement the rest of the stuff"),
        }
        self.params.clear();
        self.intermediates.clear();
    }
    fn osc_put(&mut self, byte: u8) {
        self.osc_buffer.push(byte);
    }

    /// Called when the OSC string is terminated
    fn handle_osc<H: Handler>(&mut self, handler: &mut H) {
        // pass raw bytes to the handler
        handler.handle_osc(&self.osc_buffer);
        self.osc_buffer.clear();
    }
    pub fn consume<H: Handler>(&mut self, byte: u8, handler: &mut H) {
        //C0 bytes handled first
        if is_execute(byte) {
            // This is later checked in the match statement as well but for Dcs Passthrough
            // Sicne Dcs state rarely occurs its fine to have it there
            if self.state.allows_execute() {
                handler.execute(byte);
                return;
            }
        }
        // Anywhere state
        // match byte {
        //     0x18 | 0x1a => {
        //         self.state = State::Ground;
        //         return;
        //     }
        //     0x1b => {
        //         self.state = State::Escape;
        //         return;
        //     }
        //     0x90 => {
        //         self.state = State::DcsEntry;
        //         return;
        //     }
        //     0x9d => {
        //         self.state = State::OscString;
        //         return;
        //     }
        //     0x9b => {
        //         self.state = State::CsiEntry;
        //         return;
        //     }
        //     _ => (),
        // }
        // Changed for clarity. Will test for perfomrance later
        if let Some(new_state) = anywhere_transition(byte) {
            self.state = new_state;
            return;
        }

        match self.state {
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
                0x30..=0x7e => {
                    self.state = State::Ground;
                    self.handle_esc(byte, handler);
                }
                0x20..=0x2f => self.collect_intermediate(byte),
                _ => {
                    panic!()
                }
            },
            State::OscString => match byte {
                0x07 | 0x9C => {
                    // BEL or ST terminator
                    self.state = State::Ground;
                    self.handle_osc(handler);
                }
                0x00..=0x17 | 0x19 | 0x1C..=0x1F => {
                    // control bytes in the OSC string
                    self.osc_put(byte);
                }
                0x20..=0x7F => {
                    // printable bytes, append to current OSC buffer
                    self.osc_put(byte);
                }
                _ => {
                    // ignore bytes outside as spec dictates
                }
            },
            State::CsiEntry => match byte {
                0x40..=0x7e => self.state = State::Ground,
                0x20..=0x2f => self.state = State::CsiIntermediate,
                0x3a => self.state = State::CsiIgnore,
                0x30..=0x39 => {
                    self.state = State::CsiParam;
                    self.current_param = (byte - b'0') as u16;
                }
                0x3b => {
                    self.params.push(self.current_param);
                    self.current_param = 0;
                }
                0x3c..=0x3f => {}
                _ => panic!(),
            },
            State::CsiParam => match byte {
                0x20..=0x2f => self.state = State::CsiIntermediate,
                0x3a => self.state = State::CsiIgnore,
                0x3c..=0x3f => self.state = State::CsiIgnore,
                0x30..=0x39 => self.collect_param(byte),
                0x3b => {
                    self.params.push(self.current_param);
                    self.current_param = 0;
                }
                0x40..=0x7e => {
                    self.params.push(self.current_param);
                    self.current_param = 0;
                    self.state = State::Ground;
                    self.handle_csi(byte, handler);
                }
                _ => panic!(),
            },
            State::CsiIntermediate => match byte {
                0x30..=0x3f => self.state = State::CsiIgnore,
                0x20..=0x2f => self.collect_intermediate(byte),
                0x40..=0x7e => {
                    self.params.push(self.current_param);
                    self.current_param = 0;
                    self.state = State::Ground;
                    self.handle_csi(byte, handler);
                }
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
    }
    #[inline(always)]
    pub fn collect_param(&mut self, byte: u8) {
        self.current_param = self.current_param * 10 + (byte - b'0') as u16;
    }
    #[inline(always)]
    pub fn collect_intermediate(&mut self, byte: u8) {
        self.intermediates.push(byte);
    }
}
