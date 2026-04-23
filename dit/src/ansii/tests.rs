#[cfg(test)]
mod tests {
    use crate::ansii::{Handler, Parser, State};
    use smallvec::SmallVec;

    // Every method records its call as a `Call` variant so tests can assert
    // exactly which methods were invoked and with what arguments, in order.

    #[derive(Debug, PartialEq, Clone)]
    enum Call {
        CursorUp(u16),
        CursorDown(u16),
        CursorRight(u16),
        CursorLeft(u16),
        CursorHorizontalAbsolute(u16),
        CursorVerticalAbsolute(u16),
        CursorPosition(u16, u16),
        NextLine,
        PreviousLine,
        SaveCursorPosition,
        RestoreCursorPosition,
        EraseDisplay(u16),
        EraseLine(u16),
        EraseChars(u16),
        InsertBlankChars(u16),
        DeleteChars(u16),
        InsertLines(u16),
        DeleteLines(u16),
        ScrollUp(u16),
        ScrollDown(u16),
        SetScrollingRegion(u16, u16),
        CharAttributes(Vec<u16>),
        SetTabStop,
        ClearTabStop(u16),
        CursorForwardTab(u16),
        CursorBackwardTab(u16),
        SetMode(Vec<u16>, bool),
        ResetMode(Vec<u16>, bool),
        PrimaryDeviceAttributes,
        SecondaryDeviceAttributes,
        DeviceStatusReport(u16),
        SoftReset,
        SetCursorStyle(u16),
        WindowOps(Vec<u16>),
        Index,
        ReverseIndex,
        NextLineEsc,
        SetKeypadApplicationMode,
        UnsetKeypadApplicationMode,
        Execute(u8),
        HandleOsc(Vec<u8>),
        AccumluateUtf8(u8),
        Bell,
    }

    #[derive(Default)]
    struct Mock {
        calls: Vec<Call>,
    }

    impl Handler for Mock {
        fn cursor_up(&mut self, n: u16) {
            self.calls.push(Call::CursorUp(n));
        }
        fn cursor_down(&mut self, n: u16) {
            self.calls.push(Call::CursorDown(n));
        }
        fn cursor_right(&mut self, n: u16) {
            self.calls.push(Call::CursorRight(n));
        }
        fn cursor_left(&mut self, n: u16) {
            self.calls.push(Call::CursorLeft(n));
        }
        fn cursor_horizontal_absolute(&mut self, col: u16) {
            self.calls.push(Call::CursorHorizontalAbsolute(col));
        }
        fn cursor_vertical_absolute(&mut self, row: u16) {
            self.calls.push(Call::CursorVerticalAbsolute(row));
        }
        fn cursor_position(&mut self, row: u16, col: u16) {
            self.calls.push(Call::CursorPosition(row, col));
        }
        fn next_line(&mut self) {
            self.calls.push(Call::NextLine);
        }
        fn previous_line(&mut self) {
            self.calls.push(Call::PreviousLine);
        }
        fn save_cursor_position(&mut self) {
            self.calls.push(Call::SaveCursorPosition);
        }
        fn restore_cursor_position(&mut self) {
            self.calls.push(Call::RestoreCursorPosition);
        }
        fn erase_display(&mut self, mode: u16) {
            self.calls.push(Call::EraseDisplay(mode));
        }
        fn erase_line(&mut self, mode: u16) {
            self.calls.push(Call::EraseLine(mode));
        }
        fn erase_chars(&mut self, n: u16) {
            self.calls.push(Call::EraseChars(n));
        }
        fn insert_blank_chars(&mut self, n: u16) {
            self.calls.push(Call::InsertBlankChars(n));
        }
        fn delete_chars(&mut self, n: u16) {
            self.calls.push(Call::DeleteChars(n));
        }
        fn insert_lines(&mut self, n: u16) {
            self.calls.push(Call::InsertLines(n));
        }
        fn delete_lines(&mut self, n: u16) {
            self.calls.push(Call::DeleteLines(n));
        }
        fn scroll_up(&mut self, n: u16) {
            self.calls.push(Call::ScrollUp(n));
        }
        fn scroll_down(&mut self, n: u16) {
            self.calls.push(Call::ScrollDown(n));
        }
        fn set_scrolling_region(&mut self, t: u16, b: u16) {
            self.calls.push(Call::SetScrollingRegion(t, b));
        }
        fn char_attributes(&mut self, p: &SmallVec<[u16; 8]>) {
            self.calls.push(Call::CharAttributes(p.to_vec()));
        }
        fn set_tab_stop(&mut self) {
            self.calls.push(Call::SetTabStop);
        }
        fn clear_tab_stop(&mut self, m: u16) {
            self.calls.push(Call::ClearTabStop(m));
        }
        fn cursor_forward_tab(&mut self, n: u16) {
            self.calls.push(Call::CursorForwardTab(n));
        }
        fn cursor_backward_tab(&mut self, n: u16) {
            self.calls.push(Call::CursorBackwardTab(n));
        }
        fn set_mode(&mut self, p: &SmallVec<[u16; 8]>, priv_: bool) {
            self.calls.push(Call::SetMode(p.to_vec(), priv_));
        }
        fn reset_mode(&mut self, p: &SmallVec<[u16; 8]>, priv_: bool) {
            self.calls.push(Call::ResetMode(p.to_vec(), priv_));
        }
        fn primary_device_attributes(&mut self) {
            self.calls.push(Call::PrimaryDeviceAttributes);
        }
        fn secondary_device_attributes(&mut self) {
            self.calls.push(Call::SecondaryDeviceAttributes);
        }
        fn device_status_report(&mut self, p: u16) {
            self.calls.push(Call::DeviceStatusReport(p));
        }
        fn soft_reset(&mut self) {
            self.calls.push(Call::SoftReset);
        }
        fn set_cursor_style(&mut self, s: u16) {
            self.calls.push(Call::SetCursorStyle(s));
        }
        fn window_ops(&mut self, p: &SmallVec<[u16; 8]>) {
            self.calls.push(Call::WindowOps(p.to_vec()));
        }
        fn index(&mut self) {
            self.calls.push(Call::Index);
        }
        fn reverse_index(&mut self) {
            self.calls.push(Call::ReverseIndex);
        }
        fn next_line_esc(&mut self) {
            self.calls.push(Call::NextLineEsc);
        }
        fn set_keypad_application_mode(&mut self) {
            self.calls.push(Call::SetKeypadApplicationMode);
        }
        fn unset_keypad_application_mode(&mut self) {
            self.calls.push(Call::UnsetKeypadApplicationMode);
        }
        fn execute(&mut self, b: u8) {
            self.calls.push(Call::Execute(b));
        }
        fn handle_osc(&mut self, v: &Vec<u8>) {
            self.calls.push(Call::HandleOsc(v.clone()));
        }
        fn accumluate_utf8(&mut self, b: u8) {
            self.calls.push(Call::AccumluateUtf8(b));
        }
        fn bell(&mut self) {
            self.calls.push(Call::Bell);
        }
        fn csi(&mut self) {}
    }

    fn feed(input: &[u8]) -> Mock {
        let mut parser = Parser::new();
        let mut mock = Mock::default();
        for &b in input {
            println!("Consumed byte: {:x}", b);
            parser.consume(b, &mut mock);
            println!("State: {:?}", parser.state);
        }
        mock
    }

    /// Feed bytes and also return the parser so state can be inspected.
    fn feed_with_parser(input: &[u8]) -> (Parser, Mock) {
        let mut parser = Parser::new();
        let mut mock = Mock::default();
        for &b in input {
            parser.consume(b, &mut mock);
        }
        (parser, mock)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // 1. Ground state – plain text
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn ground_printable_ascii() {
        let m = feed(b"Hello");
        assert_eq!(
            m.calls,
            b"Hello"
                .iter()
                .map(|&b| Call::AccumluateUtf8(b))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn ground_high_bytes_passed_through() {
        // UTF-8 continuation bytes (>= 0x80) should reach accumluate_utf8
        let m = feed(&[0xc3, 0xa9]); // 'é' in UTF-8
        assert_eq!(
            m.calls,
            vec![Call::AccumluateUtf8(0xc3), Call::AccumluateUtf8(0xa9),]
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // 2. C0 execute bytes
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn execute_lf() {
        let m = feed(&[0x0a]); // LF
        assert_eq!(m.calls, vec![Call::Execute(0x0a)]);
    }

    #[test]
    fn execute_cr() {
        let m = feed(&[0x0d]); // CR
        assert_eq!(m.calls, vec![Call::Execute(0x0d)]);
    }

    #[test]
    fn execute_bs() {
        let m = feed(&[0x08]); // BS
        assert_eq!(m.calls, vec![Call::Execute(0x08)]);
    }

    #[test]
    fn execute_in_csi_param_state() {
        // A C0 byte inside a CSI param sequence should still execute
        // e.g. ESC [ 1 <LF> — LF executes mid-sequence
        let m = feed(b"\x1b[1\x0aA"); // CSI 1 LF A
        assert!(m.calls.contains(&Call::Execute(0x0a)));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // 3. State transitions – verify parser.state after partial sequences
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn state_after_lone_esc() {
        let (p, _) = feed_with_parser(&[0x1b]);
        assert!(matches!(p.state, State::Escape));
    }

    #[test]
    fn state_after_csi_introducer() {
        let (p, _) = feed_with_parser(&[0x1b, b'[']);
        assert!(matches!(p.state, State::CsiEntry));
    }

    #[test]
    fn state_after_csi_digit() {
        let (p, _) = feed_with_parser(&[0x1b, b'[', b'3']);
        assert!(matches!(p.state, State::CsiParam));
    }

    #[test]
    fn state_returns_to_ground_after_csi_final() {
        let (p, _) = feed_with_parser(b"\x1b[3A");
        assert!(matches!(p.state, State::Ground));
    }

    #[test]
    fn state_after_osc_introducer() {
        let (p, _) = feed_with_parser(&[0x1b, b']']);
        assert!(matches!(p.state, State::OscString));
    }

    #[test]
    fn state_after_dcs_introducer() {
        let (p, _) = feed_with_parser(&[0x1b, b'P']);
        assert!(matches!(p.state, State::DcsEntry));
    }

    #[test]
    fn anywhere_transition_esc_resets_to_escape_from_csi_param() {
        // Mid-CSI ESC should abort and enter Escape
        let (p, _) = feed_with_parser(b"\x1b[12\x1b");
        assert!(matches!(p.state, State::Escape));
    }

    #[test]
    fn anywhere_0x18_resets_to_ground() {
        let (p, _) = feed_with_parser(&[0x1b, b'[', 0x18]);
        assert!(matches!(p.state, State::Ground));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // 4. Param collection
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn single_digit_param() {
        let m = feed(b"\x1b[3A");
        assert_eq!(m.calls, vec![Call::CursorUp(3)]);
    }

    #[test]
    fn multi_digit_param() {
        let m = feed(b"\x1b[42A");
        assert_eq!(m.calls, vec![Call::CursorUp(42)]);
    }

    #[test]
    fn zero_param_uses_default_1() {
        // ESC [ 0 A  — zero should default to 1
        let m = feed(b"\x1b[0A");
        assert_eq!(m.calls, vec![Call::CursorUp(1)]);
    }

    #[test]
    fn missing_param_uses_default_1() {
        // ESC [ A  — no param at all, defaults to 1
        let m = feed(b"\x1b[A");
        assert_eq!(m.calls, vec![Call::CursorUp(1)]);
    }

    #[test]
    fn two_params_separated_by_semicolon() {
        // ESC [ 5 ; 10 H  → cursor_position(5, 10)
        let m = feed(b"\x1b[5;10H");
        assert_eq!(m.calls, vec![Call::CursorPosition(5, 10)]);
    }

    #[test]
    fn cursor_position_defaults_to_1_1() {
        // ESC [ H  — both params omitted
        let m = feed(b"\x1b[H");
        assert_eq!(m.calls, vec![Call::CursorPosition(1, 1)]);
    }

    #[test]
    fn cursor_position_second_param_defaults() {
        // ESC [ 3 ; H  — second param omitted → 1
        let m = feed(b"\x1b[3;H");
        assert_eq!(m.calls, vec![Call::CursorPosition(3, 1)]);
    }

    #[test]
    fn params_cleared_between_sequences() {
        // Two back-to-back sequences must not bleed params into each other
        let m = feed(b"\x1b[5A\x1b[B");
        assert_eq!(
            m.calls,
            vec![
                Call::CursorUp(5),
                Call::CursorDown(1), // default, not 5
            ]
        );
    }

    #[test]
    fn many_params_sgr() {
        // SGR: ESC [ 1 ; 32 ; 40 m
        let m = feed(b"\x1b[1;32;40m");
        assert_eq!(m.calls, vec![Call::CharAttributes(vec![1, 32, 40])]);
    }

    #[test]
    fn param_overflow_saturates() {
        // A param larger than u16::MAX should saturate, not wrap or panic
        let m = feed(b"\x1b[99999A");
        assert!(matches!(m.calls[0], Call::CursorUp(_)));
        if let Call::CursorUp(n) = m.calls[0] {
            assert!(n > 0); // saturated, still usable
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // 5. Cursor movement (CSI A–G, d, e)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn cursor_up_csi_a() {
        assert_eq!(feed(b"\x1b[4A").calls, vec![Call::CursorUp(4)]);
    }

    #[test]
    fn cursor_down_csi_b() {
        assert_eq!(feed(b"\x1b[2B").calls, vec![Call::CursorDown(2)]);
    }

    #[test]
    fn cursor_right_csi_c() {
        // 'C' must be cursor_right (previously swapped bug)
        assert_eq!(feed(b"\x1b[7C").calls, vec![Call::CursorRight(7)]);
    }

    #[test]
    fn cursor_left_csi_d() {
        // 'D' must be cursor_left (previously swapped bug)
        assert_eq!(feed(b"\x1b[7D").calls, vec![Call::CursorLeft(7)]);
    }

    #[test]
    fn cursor_next_line_csi_e() {
        assert_eq!(feed(b"\x1b[E").calls, vec![Call::NextLine]);
    }

    #[test]
    fn cursor_previous_line_csi_f_upper() {
        assert_eq!(feed(b"\x1b[F").calls, vec![Call::PreviousLine]);
    }

    #[test]
    fn cursor_horizontal_absolute_csi_g_upper() {
        // 'G' is CHA — must NOT be previous_line (previously a bug)
        assert_eq!(
            feed(b"\x1b[8G").calls,
            vec![Call::CursorHorizontalAbsolute(8)]
        );
    }

    #[test]
    fn cursor_vertical_absolute_csi_d_lower() {
        assert_eq!(
            feed(b"\x1b[12d").calls,
            vec![Call::CursorVerticalAbsolute(12)]
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // 6. Erase sequences
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn erase_display_default() {
        // ESC [ J  →  ED 0 (erase below)
        assert_eq!(feed(b"\x1b[J").calls, vec![Call::EraseDisplay(0)]);
    }

    #[test]
    fn erase_display_mode_2() {
        assert_eq!(feed(b"\x1b[2J").calls, vec![Call::EraseDisplay(2)]);
    }

    #[test]
    fn erase_line_default() {
        assert_eq!(feed(b"\x1b[K").calls, vec![Call::EraseLine(0)]);
    }

    #[test]
    fn erase_line_mode_1() {
        assert_eq!(feed(b"\x1b[1K").calls, vec![Call::EraseLine(1)]);
    }

    #[test]
    fn erase_chars() {
        assert_eq!(feed(b"\x1b[5X").calls, vec![Call::EraseChars(5)]);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // 7. Insert / Delete / Scroll
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn insert_blank_chars() {
        assert_eq!(feed(b"\x1b[3@").calls, vec![Call::InsertBlankChars(3)]);
    }

    #[test]
    fn delete_chars() {
        assert_eq!(feed(b"\x1b[4P").calls, vec![Call::DeleteChars(4)]);
    }

    #[test]
    fn insert_lines() {
        assert_eq!(feed(b"\x1b[2L").calls, vec![Call::InsertLines(2)]);
    }

    #[test]
    fn delete_lines() {
        assert_eq!(feed(b"\x1b[3M").calls, vec![Call::DeleteLines(3)]);
    }

    #[test]
    fn scroll_up() {
        assert_eq!(feed(b"\x1b[2S").calls, vec![Call::ScrollUp(2)]);
    }

    #[test]
    fn scroll_down() {
        assert_eq!(feed(b"\x1b[3T").calls, vec![Call::ScrollDown(3)]);
    }

    #[test]
    fn set_scrolling_region() {
        assert_eq!(
            feed(b"\x1b[5;20r").calls,
            vec![Call::SetScrollingRegion(5, 20)]
        );
    }

    #[test]
    fn set_scrolling_region_defaults() {
        // ESC [ r  — both params default to 1 and u16::MAX respectively
        let m = feed(b"\x1b[r");
        assert!(matches!(m.calls[0], Call::SetScrollingRegion(1, _)));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // 8. SGR (Select Graphic Rendition)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn sgr_reset_empty() {
        // ESC [ m  — no params = SGR 0 reset, must NOT panic
        let m = feed(b"\x1b[m");
        assert_eq!(m.calls, vec![Call::CharAttributes(vec![0])]);
    }

    #[test]
    fn sgr_bold() {
        assert_eq!(feed(b"\x1b[1m").calls, vec![Call::CharAttributes(vec![1])]);
    }

    #[test]
    fn sgr_256_color_fg() {
        // ESC [ 38 ; 5 ; 200 m
        let m = feed(b"\x1b[38;5;200m");
        assert_eq!(m.calls, vec![Call::CharAttributes(vec![38, 5, 200])]);
    }

    #[test]
    fn sgr_rgb_color() {
        // ESC [ 38 ; 2 ; 255 ; 128 ; 0 m
        let m = feed(b"\x1b[38;2;255;128;0m");
        assert_eq!(
            m.calls,
            vec![Call::CharAttributes(vec![38, 2, 255, 128, 0])]
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // 9. Modes (SM / RM)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn set_private_mode() {
        // ESC [ ? 1 h  — DECCKM application cursor keys
        let m = feed(b"\x1b[?1h");
        assert_eq!(m.calls, vec![Call::SetMode(vec![1], true)]);
    }

    #[test]
    fn reset_private_mode() {
        let m = feed(b"\x1b[?1l");
        assert_eq!(m.calls, vec![Call::ResetMode(vec![1], true)]);
    }

    #[test]
    fn set_public_mode() {
        let m = feed(b"\x1b[4h");
        assert_eq!(m.calls, vec![Call::SetMode(vec![4], false)]);
    }

    #[test]
    fn reset_public_mode() {
        let m = feed(b"\x1b[4l");
        assert_eq!(m.calls, vec![Call::ResetMode(vec![4], false)]);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // 10. Device attributes and status
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn primary_device_attributes() {
        assert_eq!(feed(b"\x1b[c").calls, vec![Call::PrimaryDeviceAttributes]);
    }

    #[test]
    fn secondary_device_attributes() {
        assert_eq!(
            feed(b"\x1b[>c").calls,
            vec![Call::SecondaryDeviceAttributes]
        );
    }

    #[test]
    fn device_status_report() {
        // ESC [ 5 n  — operating status
        assert_eq!(feed(b"\x1b[5n").calls, vec![Call::DeviceStatusReport(5)]);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // 11. Cursor save / restore
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn csi_save_cursor_s() {
        assert_eq!(feed(b"\x1b[s").calls, vec![Call::SaveCursorPosition]);
    }

    #[test]
    fn csi_restore_cursor_u() {
        assert_eq!(feed(b"\x1b[u").calls, vec![Call::RestoreCursorPosition]);
    }

    #[test]
    fn esc_save_cursor_7() {
        assert_eq!(feed(b"\x1b7").calls, vec![Call::SaveCursorPosition]);
    }

    #[test]
    fn esc_restore_cursor_8() {
        assert_eq!(feed(b"\x1b8").calls, vec![Call::RestoreCursorPosition]);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // 12. ESC sequences (non-CSI)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn esc_index() {
        assert_eq!(feed(b"\x1bD").calls, vec![Call::Index]);
    }

    #[test]
    fn esc_reverse_index() {
        assert_eq!(feed(b"\x1bM").calls, vec![Call::ReverseIndex]);
    }

    #[test]
    fn esc_next_line() {
        assert_eq!(feed(b"\x1bE").calls, vec![Call::NextLineEsc]);
    }

    #[test]
    fn esc_set_tab_stop() {
        assert_eq!(feed(b"\x1bH").calls, vec![Call::SetTabStop]);
    }

    #[test]
    fn esc_application_keypad() {
        assert_eq!(feed(b"\x1b=").calls, vec![Call::SetKeypadApplicationMode]);
    }

    #[test]
    fn esc_normal_keypad() {
        assert_eq!(feed(b"\x1b>").calls, vec![Call::UnsetKeypadApplicationMode]);
    }

    #[test]
    fn esc_soft_reset_ris() {
        assert_eq!(feed(b"\x1bc").calls, vec![Call::SoftReset]);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // 13. DECSCUSR – cursor style
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn set_cursor_style() {
        // ESC [ 2   q  — steady block cursor
        assert_eq!(feed(b"\x1b[2 q").calls, vec![Call::SetCursorStyle(2)]);
    }

    #[test]
    fn set_cursor_style_default() {
        // ESC [   q  — no param = 0 = reset
        assert_eq!(feed(b"\x1b[ q").calls, vec![Call::SetCursorStyle(0)]);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // 14. DECSTR – soft terminal reset
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn soft_reset_csi_bang_p() {
        // ESC [ ! p
        assert_eq!(feed(b"\x1b[!p").calls, vec![Call::SoftReset]);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // 15. Tab sequences
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn cursor_forward_tab() {
        assert_eq!(feed(b"\x1b[I").calls, vec![Call::CursorForwardTab(1)]);
    }

    #[test]
    fn cursor_backward_tab() {
        assert_eq!(feed(b"\x1b[Z").calls, vec![Call::CursorBackwardTab(1)]);
    }

    #[test]
    fn clear_tab_stop_all() {
        assert_eq!(feed(b"\x1b[3g").calls, vec![Call::ClearTabStop(3)]);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // 16. OSC strings
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn osc_terminated_by_bel() {
        // OSC 0 ; title BEL  — common title-set sequence
        let seq = b"\x1b]0;my title\x07";
        let m = feed(seq);
        println!("{:?}", m.calls);
        assert_eq!(m.calls, vec![Call::HandleOsc(b"0;my title".to_vec())]);
    }

    #[test]
    fn osc_terminated_by_st() {
        // 8-bit ST (0x9C)
        let mut seq = b"\x1b]0;hello".to_vec();
        seq.push(0x9c);
        let m = feed(&seq);
        assert_eq!(m.calls, vec![Call::HandleOsc(b"0;hello".to_vec())]);
    }

    #[test]
    fn osc_empty_payload() {
        let m = feed(b"\x1b]\x07");
        assert_eq!(m.calls, vec![Call::HandleOsc(vec![])]);
    }

    #[test]
    fn osc_buffer_cleared_between_sequences() {
        let m = feed(b"\x1b]one\x07\x1b]two\x07");
        assert_eq!(
            m.calls,
            vec![
                Call::HandleOsc(b"one".to_vec()),
                Call::HandleOsc(b"two".to_vec()),
            ]
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // 17. CSI ignore path (sub-params / invalid)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn csi_colon_sub_param_is_ignored() {
        // ESC [ 38 : 2 m  — colon sub-params not supported, sequence ignored
        let m = feed(b"\x1b[38:2m");
        // No CharAttributes call should have been dispatched
        assert!(!m.calls.iter().any(|c| matches!(c, Call::CharAttributes(_))));
    }

    #[test]
    fn csi_ignore_does_not_leak_into_next_sequence() {
        // After an ignored sequence the parser must return to Ground and
        // handle the next sequence cleanly.
        let m = feed(b"\x1b[38:2m\x1b[1m");
        assert_eq!(m.calls, vec![Call::CharAttributes(vec![1])]);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // 18. DCS strings (pass-through)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn dcs_sequence_returns_to_ground_on_st() {
        // ESC P … ST (0x9C)
        let mut seq = b"\x1bP+q706978656c".to_vec();
        seq.push(0x9c);
        let (p, _) = feed_with_parser(&seq);
        assert!(matches!(p.state, State::Ground));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // 19. Intermediate bytes in CSI
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn csi_intermediate_collected_before_final() {
        // ESC [ > c  — secondary DA: intermediate '>' is 0x3e
        let m = feed(b"\x1b[>c");
        assert_eq!(m.calls, vec![Call::SecondaryDeviceAttributes]);
    }

    #[test]
    fn csi_private_marker_question_mark() {
        // '?' = 0x3f is treated as an intermediate in CsiEntry
        let m = feed(b"\x1b[?25h"); // show cursor
        assert_eq!(m.calls, vec![Call::SetMode(vec![25], true)]);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // 20. Sequences back-to-back (no stale state)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn back_to_back_cursor_moves() {
        let m = feed(b"\x1b[3A\x1b[4B\x1b[5C\x1b[6D");
        assert_eq!(
            m.calls,
            vec![
                Call::CursorUp(3),
                Call::CursorDown(4),
                Call::CursorRight(5),
                Call::CursorLeft(6),
            ]
        );
    }

    #[test]
    fn text_between_sequences() {
        let m = feed(b"hi\x1b[2Jthere");
        assert_eq!(
            m.calls,
            vec![
                Call::AccumluateUtf8(b'h'),
                Call::AccumluateUtf8(b'i'),
                Call::EraseDisplay(2),
                Call::AccumluateUtf8(b't'),
                Call::AccumluateUtf8(b'h'),
                Call::AccumluateUtf8(b'e'),
                Call::AccumluateUtf8(b'r'),
                Call::AccumluateUtf8(b'e'),
            ]
        );
    }

    #[test]
    fn esc_abort_mid_csi_then_valid_esc() {
        // ESC [ 5 ESC D  — the ESC aborts the CSI; ESC D = IND
        let m = feed(b"\x1b[5\x1bD");
        assert_eq!(m.calls, vec![Call::Index]);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // 21. 8-bit single-byte C1 introducers
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn csi_8bit_introducer_0x9b() {
        // 0x9B is the 8-bit equivalent of ESC [
        let m = feed(b"\x9b3A");
        assert_eq!(m.calls, vec![Call::CursorUp(3)]);
    }

    #[test]
    fn osc_8bit_introducer_0x9d() {
        // 0x9D is the 8-bit equivalent of ESC ]
        let mut seq = vec![0x9du8];
        seq.extend_from_slice(b"0;title");
        seq.push(0x9c); // ST
        let m = feed(&seq);
        assert_eq!(m.calls, vec![Call::HandleOsc(b"0;title".to_vec())]);
    }
}
