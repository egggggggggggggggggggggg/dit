use ash::{
    Device, Entry,
    ext::debug_utils,
    khr::{self, surface},
    vk::{self, ApplicationInfo},
};
pub mod debug;
pub mod vkcore;
use crate::render::debug::*;
use std::{
    collections::HashSet,
    ffi::{CStr, CString},
    os::raw::{c_char, c_void},
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
    raw_window_handle::HasDisplayHandle,
    window::Window,
};
const WIDTH: u32 = 960;
const HEIGHT: u32 = 540;
#[derive(Default)]
pub struct App {
    pressed_keys: HashSet<KeyCode>,
    window: Option<Window>,
}
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            println!("Creating window");
            let window_attributes = Window::default_attributes()
                .with_title("ditto")
                .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT));

            let window = event_loop.create_window(window_attributes).unwrap();
            window.request_redraw();
            self.window = Some(window);
        }
    }
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                println!("Redraw was requested");
            }
            WindowEvent::Resized(new_dimensions) => {}
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                if let PhysicalKey::Code(key) = event.physical_key {
                    match event.state {
                        ElementState::Pressed => {
                            self.pressed_keys.insert(key);
                            println!("key pressed: {:?}", key);
                        }
                        ElementState::Released => {
                            self.pressed_keys.remove(&key);
                        }
                    }
                }
            }
            _ => (),
        }
    }
}
impl App {}
