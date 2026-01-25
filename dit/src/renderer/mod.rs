mod buffer;
mod command;
mod context;
mod debug;
mod device;
mod pipeline;
mod queue;
mod renderpass;
mod resources;
mod shader;
mod swapchain;
mod texture;
mod utils;
mod vkapp;
// mod vkcore;
use ash::vk;
use buffer::*;
use command::*;
use context::*;
use debug::*;
use device::*;
use pipeline::*;
use queue::*;
use renderpass::*;
use resources::*;
use shader::*;
use swapchain::*;
use texture::*;
use utils::*;
// use vkcore::*;

use std::{collections::HashSet, time::Instant};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use crate::{renderer::vkapp::VkApplication, screen::Screen};
const WIDTH: u32 = 960;
const HEIGHT: u32 = 540;
//cannot use default for this as some config info is required 
#[derive(Default)]
pub struct App {
    pressed_keys: HashSet<KeyCode>,
    window: Option<Window>,
    pub vk_app: Option<VkApplication>,
    frame_counter: u32,
    instant: Option<Instant>,
    screen: Screen,
}
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_title("Vulkan tutorial with Ash")
                    .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT)),
            )
            .unwrap();
        println!("Inner window size: {:?}", window.inner_size());
        self.vk_app = Some(VkApplication::new(&window, 10));
        self.window = Some(window);
        self.instant = Some(Instant::now());
        self.screen = Screen::new(10, 30);
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
            WindowEvent::Resized(new_dimensions) => {
                println!("new dimensions to resize to: {:?}", new_dimensions)
            }
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
                self.screen.change(&event, &self.pressed_keys);
            }
            _ => (),
        }
    }
    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        let app = self.vk_app.as_mut().unwrap();
        let window = self.window.as_ref().unwrap();
        if app.dirty_swapchain {
            let size = window.inner_size();
            if size.width > 0 && size.height > 0 {
                app.resize_dimensions = [size.width, size.height];
                app.recreate_swapchain();
            } else {
                return;
            }
        }
        app.dirty_swapchain = app.draw_frame();
        if let Some(instant) = self.instant {
            if instant.elapsed().as_millis() >= 1000 {
                self.instant = Some(Instant::now());
                println!("FPS: {}", self.frame_counter);
                self.frame_counter = 0;
            }
        }
        self.frame_counter += 1;
    }
    fn exiting(&mut self, _: &ActiveEventLoop) {
        self.vk_app.as_ref().unwrap().wait_gpu_idle();
    }
}
//for each frame give
