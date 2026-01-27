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
use atlas_gen::{allocator::ShelfAllocator, atlas::Atlas};
use buffer::*;
use command::*;
use context::*;
use debug::*;
use device::*;
use image::Rgb;
use pipeline::*;
use queue::*;
use renderpass::*;
use resources::*;
use shader::*;
use swapchain::*;
use texture::*;
use utils::*;
// use vkcore::*;

use std::{cell, collections::HashSet, time::Instant};
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
pub struct App {
    pressed_keys: HashSet<KeyCode>,
    window: Option<Window>,
    pub vk_app: Option<VkApplication>,
    frame_counter: u32,
    instant: Option<Instant>,
    screen: Screen,
    glyph_mesh: Option<Mesh>,
    pub atlas: Atlas<char, Rgb<u8>, ShelfAllocator>,
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
        self.vk_app = Some(VkApplication::new(
            &window,
            &self.glyph_mesh.clone().unwrap(),
        ));
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
                // self.vk_app.as_mut().unwrap().test_dynamic_buffer();
            }
        }
        self.frame_counter += 1;
    }
    fn exiting(&mut self, _: &ActiveEventLoop) {
        self.vk_app.as_ref().unwrap().wait_gpu_idle();
    }
}
//for each frame give
pub fn get_uv(char: Option<char>) -> [f32; 4] {
    [0.0f32, 0.0f32, 0.0f32, 0.0f32]
}
impl App {
    pub fn new(row_size: usize, cell_size: usize) -> Self {
        let atlas = atlas_gen::entry();
        Self {
            frame_counter: 0,
            glyph_mesh: None,
            screen: Screen::new(row_size, cell_size),
            instant: Some(Instant::now()),
            pressed_keys: HashSet::new(),
            window: None,
            vk_app: None,
            atlas,
        }
    }
    // //this should really have each row have its own instance data instead so we dont reconstruct the whole thing
    // pub fn construct_mesh(&self) -> Vec<InstanceData> {
    //     let rows = &self.screen.rows;
    //     let mut instance_vec = Vec::new();
    //     let mut row_counter = 0.0f32;
    //     for row in rows {
    //         let mut cell_counter = 0.0f32;
    //         for cell in &row.cells {
    //             let instance_data = InstanceData {
    //                 pos: [cell_counter, row_counter],
    //                 size: [1.0, 1.0],
    //                 uv: get_uv(cell.glyph),
    //                 bg: cell.bg,
    //                 fg: cell.fg,
    //             };
    //             instance_vec.push(instance_data);
    //             cell_counter += 1.0;
    //         }
    //         row_counter += 1.0;
    //     }
    //     instance_vec
    // }
    // pub fn construct_row_mesh(&mut self, row_number: usize) {
    //     let row = &self.screen.rows[row_number];
    //     let mut cell_counter = 0.0f32;
    //     let glyph_mesh = &mut self.glyph_mesh;
    //     for cell in &row.cells {
    //         let instance_data = InstanceData {
    //             pos: [cell_counter, row_number as f32],
    //             size: [1.0, 1.0],
    //             uv: get_uv(cell.glyph),
    //             bg: cell.bg,
    //             fg: cell.fg,
    //         };
    //         let offset = self.screen.max_cells * row_number + cell_counter as usize;
    //         glyph_mesh[offset] = instance_data;
    //         cell_counter += 1.0;
    //     }
    // }
    pub fn check_damage(&mut self) -> bool {
        let screen = &self.screen;
        if screen.damaged.len() == 0 {
            return false;
        }
        let cell_width = 2.0 / screen.max_cells as f32;
        let cell_height = 2.0 / screen.max_rows as f32;
        let mesh = self.glyph_mesh.as_mut().unwrap();
        while let Some(row_idx) = screen.damaged.clone().pop() {
            let row = &screen.rows[row_idx];
            let mut mesh_vertices_offset = screen.max_cells * 4;
            for (col_idx, cell) in row.cells.iter().enumerate() {
                let x0 = -1.0 + col_idx as f32 * cell_width;
                let y0 = 1.0 - (row_idx + 1) as f32 * cell_height;
                let x1 = x0 + cell_width;
                let y1 = y0 + cell_height;
                let ([u0, v0], [u1, v1]) = match cell.glyph {
                    None => ([0.0, 0.0], [0.0, 0.0]),
                    Some(char) => self.atlas.get_uv(char),
                };
                mesh.vertices[mesh_vertices_offset] = Vertex {
                    pos: [x0, y0],
                    uv: [u0, v0],
                };
                mesh.vertices[mesh_vertices_offset + 1] = Vertex {
                    pos: [x0, y1],
                    uv: [u0, v1],
                };
                mesh.vertices[mesh_vertices_offset + 2] = Vertex {
                    pos: [x1, y1],
                    uv: [u1, v1],
                };
                mesh.vertices[mesh_vertices_offset + 3] = Vertex {
                    pos: [x1, y0],
                    uv: [u1, v0],
                };
                mesh_vertices_offset += 4;
            }
        }
        true
    }
    pub fn generate_screen_mesh(&mut self) {
        let screen = &self.screen;
        let cell_width = 2.0 / screen.max_cells as f32; // NDC space: -1..1
        let cell_height = 2.0 / screen.max_rows as f32;
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut index_offset = 0u32;

        for (row_idx, row) in screen.rows.iter().enumerate() {
            for (col_idx, cell) in row.cells.iter().enumerate() {
                let x0 = -1.0 + col_idx as f32 * cell_width;
                let y0 = 1.0 - (row_idx + 1) as f32 * cell_height;
                let x1 = x0 + cell_width;
                let y1 = y0 + cell_height;
                let ([u0, v0], [u1, v1]) = match cell.glyph {
                    None => ([0.0, 0.0], [0.0, 0.0]),
                    Some(char) => self.atlas.get_uv(char),
                };
                vertices.push(Vertex {
                    pos: [x0, y0],
                    uv: [u0, v0],
                });
                vertices.push(Vertex {
                    pos: [x0, y1],
                    uv: [u0, v1],
                });
                vertices.push(Vertex {
                    pos: [x1, y1],
                    uv: [u1, v1],
                });
                vertices.push(Vertex {
                    pos: [x1, y0],
                    uv: [u1, v0],
                });
                indices.push(index_offset + 0);
                indices.push(index_offset + 1);
                indices.push(index_offset + 2);
                indices.push(index_offset + 2);
                indices.push(index_offset + 3);
                indices.push(index_offset + 0);
                index_offset += 4;
            }
            println!(
                "vertices length: {}, indices length: {}",
                vertices.len(),
                indices.len()
            );
        }
        let mesh = Mesh { vertices, indices };
        self.glyph_mesh = Some(mesh);
    }
}

#[derive(Clone)]
pub struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}
