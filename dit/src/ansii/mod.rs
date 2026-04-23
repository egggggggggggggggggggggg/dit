use smallvec::SmallVec;

pub mod details;
pub mod tests;
pub mod utf_decoder;
#[repr(u8)]
pub enum ESC {
    IND = b'D',
    NEL = b'E',
    HTS = b'H',
    RI = b'M',
    SS2 = b'N',
    SS3 = b'O',
    DCS = b'P',
    SPA = b'V',
    EPA = b'W',
    SOS = b'X',
    DECID = b'Z',
    CSI = b'[',
    ST = b'\\',
    OSC = b']',
    PM = b'^',
    APC = b'_',
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

pub trait Handler {
    fn cursor_up(&mut self, n: u16);
    fn cursor_down(&mut self, n: u16);
    fn cursor_right(&mut self, n: u16);
    fn cursor_left(&mut self, n: u16);
    fn cursor_horizontal_absolute(&mut self, col: u16);
    /// VPA – move to row (1-based)
    fn cursor_vertical_absolute(&mut self, row: u16);
    /// CUP / HVP – move to (row, col), both 1-based
    fn cursor_position(&mut self, row: u16, col: u16);
    /// CNL – cursor next line
    fn next_line(&mut self);
    /// CPL – cursor previous line
    fn previous_line(&mut self);
    /// DECSC / ESC 7 – save cursor + attributes
    fn save_cursor_position(&mut self);
    /// DECRC / ESC 8 – restore cursor + attributes
    fn restore_cursor_position(&mut self);

    /// ED  – erase in display  (0=below, 1=above, 2=all, 3=saved lines)
    fn erase_display(&mut self, mode: u16);
    /// EL  – erase in line     (0=right, 1=left, 2=all)
    fn erase_line(&mut self, mode: u16);
    /// ECH – erase n characters at cursor
    fn erase_chars(&mut self, n: u16);
    /// ICH – insert n blank characters
    fn insert_blank_chars(&mut self, n: u16);
    /// DCH – delete n characters
    fn delete_chars(&mut self, n: u16);
    /// IL  – insert n lines
    fn insert_lines(&mut self, n: u16);
    /// DL  – delete n lines
    fn delete_lines(&mut self, n: u16);
    /// SU  – scroll up n lines
    fn scroll_up(&mut self, n: u16);
    /// SD  – scroll down n lines
    fn scroll_down(&mut self, n: u16);
    /// DECSTBM – set scrolling region [top, bottom] (1-based, inclusive)
    fn set_scrolling_region(&mut self, top: u16, bottom: u16);
    fn char_attributes(&mut self, params: &SmallVec<[u16; 8]>);
    /// HTS / ESC H – set tab stop at current column
    fn set_tab_stop(&mut self);
    /// TBC – clear tab stops (0=current col, 3=all)
    fn clear_tab_stop(&mut self, mode: u16);
    /// CHT – cursor forward tabulation n tab stops
    fn cursor_forward_tab(&mut self, n: u16);
    /// CBT – cursor backward tabulation n tab stops
    fn cursor_backward_tab(&mut self, n: u16);
    /// SM / RM – set/reset mode.  `private` = true when '?' intermediate present
    fn set_mode(&mut self, params: &SmallVec<[u16; 8]>, private: bool);
    fn reset_mode(&mut self, params: &SmallVec<[u16; 8]>, private: bool);
    /// DA1  – primary device attributes
    fn primary_device_attributes(&mut self);
    /// DA2  – secondary device attributes (intermediate b'>')
    fn secondary_device_attributes(&mut self);
    /// DSR  – device status report
    fn device_status_report(&mut self, param: u16);
    /// DECSTR – soft terminal reset
    fn soft_reset(&mut self);
    /// DECSCUSR – set cursor style (0/1=blinking block, 2=steady block, …)
    fn set_cursor_style(&mut self, style: u16);
    /// XTWINOPS – window manipulation
    fn window_ops(&mut self, params: &SmallVec<[u16; 8]>);
    /// Index (IND) – like LF but doesn't move to column 1
    fn index(&mut self);
    /// Reverse index (RI) – scroll down if at top margin
    fn reverse_index(&mut self);
    /// NEL – move to next line *and* column 1
    fn next_line_esc(&mut self);
    /// DECKPAM / DECKPNM – application / normal keypad mode
    fn set_keypad_application_mode(&mut self);
    fn unset_keypad_application_mode(&mut self);

    fn execute(&mut self, ctl_seq: u8);
    fn handle_osc(&mut self, osc: &Vec<u8>);
    fn accumluate_utf8(&mut self, byte: u8);
    fn bell(&mut self);
    fn csi(&mut self) {}
}

// ─── Parser ──────────────────────────────────────────────────────────────────

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

/// Return the first param (or `default` when absent / zero).
/// Most CSI sequences treat an omitted or zero param as meaning "1".
#[inline(always)]
fn p(params: &SmallVec<[u16; 8]>, idx: usize, default: u16) -> u16 {
    match params.get(idx) {
        Some(&0) | None => default,
        Some(&v) => v,
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

    fn handle_esc<H: Handler>(&mut self, final_byte: u8, handler: &mut H) {
        match self.intermediates.first() {
            // ESC <intermediate> <final> – two-character escape sequences
            Some(&b'(') | Some(&b')') | Some(&b'*') | Some(&b'+') => {
                // Character-set designation – ignored for now; a full
                // implementation would update the G0/G1/G2/G3 slot.
            }
            Some(&b'#') => {
                match final_byte {
                    b'8' => { /* DECALN – screen alignment test */ }
                    _ => {}
                }
            }
            Some(&b' ') => {
                match final_byte {
                    b'F' => { /* S7C1T – 7-bit controls */ }
                    b'G' => { /* S8C1T – 8-bit controls */ }
                    b'L' => { /* ANSI conformance level 1 */ }
                    b'M' => { /* ANSI conformance level 2 */ }
                    b'N' => { /* ANSI conformance level 3 */ }
                    _ => {}
                }
            }
            None => {
                // Single-character ESC sequences (ESC <final>)
                match final_byte {
                    b'7' => handler.save_cursor_position(),
                    b'8' => handler.restore_cursor_position(),
                    b'=' => handler.set_keypad_application_mode(),
                    b'>' => handler.unset_keypad_application_mode(),
                    b'D' => handler.index(),
                    b'E' => handler.next_line_esc(),
                    b'H' => handler.set_tab_stop(),
                    b'M' => handler.reverse_index(),
                    b'c' => handler.soft_reset(), // RIS – reset to initial state
                    b'n' => { /* LS2 – locking shift G2 */ }
                    b'o' => { /* LS3 – locking shift G3 */ }
                    b'|' => { /* LS3R */ }
                    b'}' => { /* LS2R */ }
                    b'~' => { /* LS1R */ }
                    _ => {
                        eprintln!("[parser] unhandled ESC {:?}", final_byte as char);
                    }
                }
            }
            Some(i) => {
                eprintln!(
                    "[parser] unhandled ESC intermediate=0x{:02x} final={:?}",
                    i, final_byte as char
                );
            }
        }
        self.intermediates.clear();
    }

    // ── CSI dispatcher ───────────────────────────────────────────────────────

    fn handle_csi<H: Handler>(&mut self, final_byte: u8, handler: &mut H) {
        let inter = self.intermediates.first().copied();

        match final_byte {
            // ── Cursor movement ───────────────────────────────────────────
            b'A' => handler.cursor_up(p(&self.params, 0, 1)),
            b'B' => handler.cursor_down(p(&self.params, 0, 1)),
            b'C' => handler.cursor_right(p(&self.params, 0, 1)), // was swapped
            b'D' => handler.cursor_left(p(&self.params, 0, 1)),  // was swapped
            b'E' => handler.next_line(),
            b'F' => handler.previous_line(),
            b'G' => handler.cursor_horizontal_absolute(p(&self.params, 0, 1)),
            b'H' | b'f' => {
                // CUP / HVP – both use (row, col), default 1
                handler.cursor_position(p(&self.params, 0, 1), p(&self.params, 1, 1))
            }
            b'd' => handler.cursor_vertical_absolute(p(&self.params, 0, 1)),
            b'e' => handler.cursor_down(p(&self.params, 0, 1)), // VPR

            // ── Tab ───────────────────────────────────────────────────────
            b'I' => handler.cursor_forward_tab(p(&self.params, 0, 1)),
            b'Z' => handler.cursor_backward_tab(p(&self.params, 0, 1)),
            b'g' => handler.clear_tab_stop(p(&self.params, 0, 0)),

            // ── Erase ─────────────────────────────────────────────────────
            b'J' => {
                // '?' intermediate = selective erase (DECSED) — treat same as ED
                handler.erase_display(p(&self.params, 0, 0))
            }
            b'K' => handler.erase_line(p(&self.params, 0, 0)),
            b'X' => handler.erase_chars(p(&self.params, 0, 1)),

            // ── Insert / Delete ───────────────────────────────────────────
            b'@' => handler.insert_blank_chars(p(&self.params, 0, 1)),
            b'L' => handler.insert_lines(p(&self.params, 0, 1)),
            b'M' => handler.delete_lines(p(&self.params, 0, 1)),
            b'P' => handler.delete_chars(p(&self.params, 0, 1)),

            // ── Scroll ────────────────────────────────────────────────────
            b'S' => handler.scroll_up(p(&self.params, 0, 1)),
            b'T' => {
                match inter {
                    Some(b'>') => { /* xterm mouse tracking reset — ignored */ }
                    _ => handler.scroll_down(p(&self.params, 0, 1)),
                }
            }

            // ── SGR ───────────────────────────────────────────────────────
            b'm' => {
                // Empty params is a valid SGR 0 (reset) – do not panic.
                if self.params.is_empty() {
                    self.params.push(0);
                }
                handler.char_attributes(&self.params);
            }

            // ── Modes ─────────────────────────────────────────────────────
            b'h' => {
                let private = inter == Some(b'?');
                handler.set_mode(&self.params, private);
            }
            b'l' => {
                let private = inter == Some(b'?');
                handler.reset_mode(&self.params, private);
            }

            // ── Device status / identification ────────────────────────────
            b'n' => match inter {
                Some(b'>') => { /* xterm name/version report */ }
                Some(b'?') => { /* DECDSR */ }
                _ => handler.device_status_report(p(&self.params, 0, 0)),
            },
            b'c' => match inter {
                Some(b'>') => handler.secondary_device_attributes(),
                _ => handler.primary_device_attributes(),
            },

            // ── Cursor style / state ──────────────────────────────────────
            b'q' => match inter {
                Some(b' ') => handler.set_cursor_style(p(&self.params, 0, 0)),
                _ => { /* DECLL – load LEDs */ }
            },
            b'r' => match inter {
                Some(b'?') => handler.restore_cursor_position(), // DECCARA variant
                _ => {
                    // DECSTBM – set scrolling region
                    handler.set_scrolling_region(
                        p(&self.params, 0, 1),
                        p(&self.params, 1, u16::MAX), // handler must clamp to screen height
                    )
                }
            },
            b's' => handler.save_cursor_position(),
            b'u' => handler.restore_cursor_position(),

            // ── Window ops ────────────────────────────────────────────────
            b't' => handler.window_ops(&self.params),

            // ── Soft reset ───────────────────────────────────────────────
            b'p' => match inter {
                Some(b'!') => handler.soft_reset(), // DECSTR
                Some(b'"') => { /* DECSCL – conformance level */ }
                Some(b'$') => { /* DECRQM  – request mode */ }
                _ => {}
            },

            // ── VT sequences dispatched by first param (b'~') ─────────────
            b'~' => {
                // Only index if we actually have an intermediate byte
                match inter {
                    Some(b) => eprintln!("[parser] CSI ~ with intermediate 0x{:02x}", b),
                    None => {
                        // Standard VT220 function key codes
                        match p(&self.params, 0, 0) {
                            1 => { /* Home */ }
                            2 => { /* Insert */ }
                            3 => { /* Delete */ }
                            4 => { /* End */ }
                            5 => { /* Page Up */ }
                            6 => { /* Page Down */ }
                            11 => { /* F1 */ }
                            12 => { /* F2 */ }
                            13 => { /* F3 */ }
                            14 => { /* F4 */ }
                            15 => { /* F5 */ }
                            17 => { /* F6 */ }
                            18 => { /* F7 */ }
                            19 => { /* F8 */ }
                            20 => { /* F9 */ }
                            21 => { /* F10 */ }
                            23 => { /* F11 */ }
                            24 => { /* F12 */ }
                            n => eprintln!("[parser] unhandled CSI {} ~", n),
                        }
                    }
                }
            }

            b => eprintln!(
                "[parser] unhandled CSI {:?} params={:?} inter={:?}",
                b as char, self.params, self.intermediates
            ),
        }

        self.params.clear();
        self.intermediates.clear();
        self.current_param = 0;
    }

    fn osc_put(&mut self, byte: u8) {
        self.osc_buffer.push(byte);
    }

    fn handle_osc<H: Handler>(&mut self, handler: &mut H) {
        handler.handle_osc(&self.osc_buffer);
        self.osc_buffer.clear();
    }

    pub fn consume<H: Handler>(&mut self, byte: u8, handler: &mut H) {
        match self.state {
            State::OscString => match byte {
                0x07 => {
                    self.handle_osc(handler);
                    self.state = State::Ground;
                    return;
                }
                _ => {}
            },
            _ => {}
        }
        // C0 control bytes — execute in most states
        if is_execute(byte) {
            if self.state.allows_execute() {
                handler.execute(byte);
                return;
            }
            // In DCS passthrough we could forward, but we ignore for now
            return;
        }

        // Anywhere transitions take priority over the current state
        if let Some(new_state) = anywhere_transition(byte) {
            // Entering Escape from any state: clear accumulated data
            if matches!(new_state, State::Escape) {
                self.params.clear();
                self.intermediates.clear();
                self.current_param = 0;
            }
            self.state = new_state;
            return;
        }

        match self.state {
            // ── Ground ────────────────────────────────────────────────────
            State::Ground => match byte {
                0x20..=0x7f => handler.accumluate_utf8(byte),
                0x80..=0xff => handler.accumluate_utf8(byte), // high bytes (UTF-8 continuations / GR)
                _ => {}
            },

            // ── Escape ────────────────────────────────────────────────────
            State::Escape => match byte {
                0x5b => {
                    // '['
                    self.state = State::CsiEntry;
                    self.params.clear();
                    self.intermediates.clear();
                    self.current_param = 0;
                }
                0x5d => self.state = State::OscString, // ']'
                0x50 => self.state = State::DcsEntry,  // 'P'
                0x20..=0x2f => {
                    self.intermediates.push(byte);
                    self.state = State::EscapeIntermediate;
                }
                0x30..=0x4f | 0x51..=0x5a | 0x5c..=0x7e => {
                    // Final byte of single-character ESC sequence
                    self.state = State::Ground;
                    self.handle_esc(byte, handler);
                }
                _ => {
                    eprintln!("[parser] unrecognised byte 0x{:02x} in Escape state", byte);
                    self.state = State::Ground;
                }
            },

            // ── EscapeIntermediate ────────────────────────────────────────
            State::EscapeIntermediate => match byte {
                0x20..=0x2f => self.collect_intermediate(byte),
                0x30..=0x7e => {
                    self.state = State::Ground;
                    self.handle_esc(byte, handler);
                }
                _ => {
                    eprintln!(
                        "[parser] unrecognised byte 0x{:02x} in EscapeIntermediate",
                        byte
                    );
                    self.state = State::Ground;
                }
            },

            // ── OscString ────────────────────────────────────────────────
            State::OscString => match byte {
                0x9c => {
                    // BEL or ST – terminate OSC
                    self.state = State::Ground;
                    self.handle_osc(handler);
                }
                0x1b => {
                    // ESC \ (ST via two bytes) — the ESC byte itself is
                    // caught by anywhere_transition above and transitions
                    // us to State::Escape; the next byte '\\' = 0x5c will
                    // be a no-op ESC final that we can detect there.
                    // Handle as terminator right here for safety.
                    self.state = State::Ground;
                    self.handle_osc(handler);
                }
                0x20..=0x7f | 0x80..=0xff => self.osc_put(byte),
                _ => { /* ignore other control bytes inside OSC */ }
            },

            // ── CsiEntry ──────────────────────────────────────────────────
            State::CsiEntry => match byte {
                // Immediately a final byte (no params at all)
                0x40..=0x7e => {
                    // Push the (zero) current_param so handle_csi sees at
                    // least one entry for sequences that need params[0].
                    self.params.push(self.current_param);
                    self.current_param = 0;
                    self.state = State::Ground;
                    self.handle_csi(byte, handler);
                }
                0x20..=0x2f => {
                    self.collect_intermediate(byte);
                    self.state = State::CsiIntermediate;
                }
                0x3a => self.state = State::CsiIgnore, // ':' — sub-params not supported
                0x30..=0x39 => {
                    // First digit of first parameter
                    self.current_param = (byte - b'0') as u16;
                    self.state = State::CsiParam;
                }
                0x3b => {
                    // ';' before any digit — implicit leading 0
                    self.params.push(0);
                    self.current_param = 0;
                    self.state = State::CsiParam;
                }
                0x3c..=0x3f => {
                    // Private-use marker: '<', '=', '>', '?'
                    self.collect_intermediate(byte);
                    self.state = State::CsiParam;
                }
                _ => {
                    eprintln!("[parser] unexpected 0x{:02x} in CsiEntry", byte);
                    self.state = State::CsiIgnore;
                }
            },

            // ── CsiParam ──────────────────────────────────────────────────
            State::CsiParam => match byte {
                0x30..=0x39 => self.collect_param(byte),
                0x3b => {
                    // ';' – parameter separator
                    self.params.push(self.current_param);
                    self.current_param = 0;
                }
                0x3a => self.state = State::CsiIgnore, // ':' sub-param
                0x3c..=0x3f => self.state = State::CsiIgnore, // private after params
                0x20..=0x2f => {
                    self.collect_intermediate(byte);
                    self.state = State::CsiIntermediate;
                }
                0x40..=0x7e => {
                    self.params.push(self.current_param);
                    self.current_param = 0;
                    self.state = State::Ground;
                    self.handle_csi(byte, handler);
                }
                _ => {
                    eprintln!("[parser] unexpected 0x{:02x} in CsiParam", byte);
                    self.state = State::CsiIgnore;
                }
            },

            // ── CsiIntermediate ───────────────────────────────────────────
            State::CsiIntermediate => match byte {
                0x20..=0x2f => self.collect_intermediate(byte),
                0x30..=0x3f => self.state = State::CsiIgnore, // digit after intermediate
                0x40..=0x7e => {
                    self.params.push(self.current_param);
                    self.current_param = 0;
                    self.state = State::Ground;
                    self.handle_csi(byte, handler);
                }
                _ => self.state = State::Ground,
            },

            // ── CsiIgnore ─────────────────────────────────────────────────
            State::CsiIgnore => match byte {
                0x40..=0x7e => {
                    // Consume until final byte, then return to ground
                    self.params.clear();
                    self.intermediates.clear();
                    self.current_param = 0;
                    self.state = State::Ground;
                }
                _ => { /* keep ignoring */ }
            },

            // ── DcsEntry ──────────────────────────────────────────────────
            State::DcsEntry => match byte {
                0x20..=0x2f => {
                    self.collect_intermediate(byte);
                    self.state = State::DcsIntermediate;
                }
                0x40..=0x7e => self.state = State::DcsPassthrough,
                0x3a => self.state = State::DcsIgnore,
                0x30..=0x39 | 0x3b | 0x3c..=0x3f => self.state = State::DcsParam,
                _ => self.state = State::DcsIgnore,
            },

            // ── DcsParam ──────────────────────────────────────────────────
            State::DcsParam => match byte {
                0x30..=0x39 => self.collect_param(byte),
                0x3b => {
                    self.params.push(self.current_param);
                    self.current_param = 0;
                }
                0x3a | 0x3c..=0x3f => self.state = State::DcsIgnore,
                0x20..=0x2f => {
                    self.collect_intermediate(byte);
                    self.state = State::DcsIntermediate;
                }
                0x40..=0x7e => self.state = State::DcsPassthrough,
                _ => self.state = State::DcsIgnore,
            },

            // ── DcsIntermediate ───────────────────────────────────────────
            State::DcsIntermediate => match byte {
                0x20..=0x2f => self.collect_intermediate(byte),
                0x30..=0x3f => self.state = State::DcsIgnore,
                0x40..=0x7e => self.state = State::DcsPassthrough,
                _ => self.state = State::DcsIgnore,
            },

            // ── DcsPassthrough ────────────────────────────────────────────
            State::DcsPassthrough => match byte {
                0x9c | 0x1b => {
                    // ST or ESC \ — end of DCS; ESC is also caught by
                    // anywhere_transition, so 0x9c handles the 8-bit path.
                    self.params.clear();
                    self.intermediates.clear();
                    self.current_param = 0;
                    self.state = State::Ground;
                }
                _ => { /* forward to DCS handler if implemented */ }
            },

            // ── DcsIgnore ─────────────────────────────────────────────────
            State::DcsIgnore => match byte {
                0x9c => self.state = State::Ground,
                _ => { /* absorb */ }
            },
        }
    }

    #[inline(always)]
    pub fn collect_param(&mut self, byte: u8) {
        self.current_param = self
            .current_param
            .saturating_mul(10)
            .saturating_add((byte - b'0') as u16);
    }

    #[inline(always)]
    pub fn collect_intermediate(&mut self, byte: u8) {
        if self.intermediates.len() < 4 {
            self.intermediates.push(byte);
        }
    }
}
