use std::{
    collections::{HashMap, HashSet},
    time::Instant,
};

use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, ElementState, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use crate::{
    ansii::{Handler, Parser, details::Attributes, utf_decoder::Utf8Decoder},
    shell::{MarkerMatcher, Pty},
};
// In seconds
const BLINK_DURATION: f64 = 0.5;
#[derive(Debug, Default, Clone)]
pub struct Cursor {
    pub row: usize,
    pub col: usize,
    pub visible: bool,
    pub blinking: bool,
}
#[derive(Default)]
struct Cell {
    ch: char,
    cell_attr: Attributes,
}
struct Mode {
    
}
struct Screen {
    cells: Vec<Cell>,
    cursor: Cursor,
    row_size: usize,
    col_size: usize,
    accumulator: Utf8Decoder,
}
impl Screen {
    fn construct_mesh(&mut self) {}
    fn resize(&mut self, new_row: usize, new_col: usize) {
        self.col_size = new_col;
        self.row_size = new_row;
        self.cells.resize_with(new_col * new_row, Default::default);
    }
}

impl Handler for Screen {
    fn cursor_up(&mut self, n: u16) {
        self.cursor.row = self.cursor.row.saturating_sub(n as usize);
    }
    fn cursor_down(&mut self, n: u16) {
        let max_row = self.row_size - 1;
        self.cursor.row = (self.cursor.row + n as usize).min(max_row);
    }
    fn cursor_right(&mut self, n: u16) {
        let max_col = self.col_size - 1;
        self.cursor.col = self.cursor.col.saturating_add(n as usize).min(max_col);
    }

    fn cursor_left(&mut self, n: u16) {
        self.cursor.col = self.cursor.col.saturating_sub(n as usize);
    }
    fn accumluate_utf8(&mut self, byte: u8) {
        if let Some(ch) = self.accumulator.decode(byte) {
            //
            // writes the character to the visual buffer
        }
    }
    fn bell() {}
    fn char_attributes(&mut self, params: &smallvec::SmallVec<[u16; 8]>) {}
    fn csi() {}

    fn cursor_position(&mut self, new_x: u16, new_y: u16) {}
    fn device_status_report(&mut self, param: u16) {}
    fn execute(&mut self, ctl_seq: u8) {}
    fn handle_osc(&mut self, u: &Vec<u8>) {}
    fn next_line(&mut self) {}
    fn previous_line(&mut self) {}
}

struct Application {
    screen: Screen,
    pressed_keys: HashSet<KeyCode>,
    input_buffer: String,
    window: Window,
    last_frame: Instant,
    last_blink: Instant,
    pty: Pty,
    parser: Parser,
}

impl Application {
    fn update(&mut self, delta: f64) {
        if self.pressed_keys.contains(&KeyCode::ControlLeft) {
            // handle key events
        }
        if self.last_blink.elapsed().as_secs_f64() <= 0.5 {
            self.screen.cursor.visible = true;
        }
        let mut commands_to_send = Vec::new();
        while let Some(pos) = self.input_buffer.chars().position(|c| c == '\n') {
            let cmd: String = self.input_buffer.drain(..pos).collect();
            self.input_buffer.drain(..1); // remove newline
            commands_to_send.push(cmd);
        }
        self.pty.write(&commands_to_send).unwrap();
        if self.pty.poll(0).unwrap() {
            // Checks for data to read from
            let mut buf = [0u8; 4096];
            // Staging buffer is created to handle the bytes
            let n = self.pty.read(&mut buf).unwrap();
            // Read writes into said buffer
            if n != 0 {
                // Checks for if there are bytes to consume
                for byte in buf {
                    self.parser.consume(byte, &mut self.screen);
                }
            }
        }
    }
    fn draw(&mut self) {
        // responsible for the generation of a new mesh
    }
}
impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {}
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = now - self.last_frame;
                self.last_frame = now;
                self.update(dt.as_secs_f64());
            }
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                if let PhysicalKey::Code(key) = event.physical_key {
                    match event.state {
                        ElementState::Pressed => {
                            if let Some(text) = &event.text {
                                self.input_buffer.push_str(text);
                            }
                            // When a user is holding a key it still generates a Pressed event
                            if !event.repeat {
                                self.pressed_keys.insert(key);
                            }
                        }
                        ElementState::Released => {
                            self.pressed_keys.remove(&key);
                        }
                    }
                }
            }
            _ => {}
        }
    }
    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.window.request_redraw();
    }
    fn device_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        match event {
            DeviceEvent::MouseWheel { delta } => {}
            _ => {}
        }
    }
}
