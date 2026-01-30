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
use font_parser::TtfFont;
use image::Rgb;
use pipeline::*;
use queue::*;
use rand::prelude::*;
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
    pub ttf_font: TtfFont,
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
        let ttf_font = TtfFont::new("../JetBrainsMonoNerdFontMono-Regular.ttf").unwrap();
        Self {
            frame_counter: 0,
            glyph_mesh: None,
            screen: Screen::new(row_size, cell_size),
            instant: Some(Instant::now()),
            pressed_keys: HashSet::new(),
            window: None,
            vk_app: None,
            atlas,
            ttf_font,
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
    //Inlinining evryething for now for testing purposes, will swap to make it more modular
    pub fn generate_screen_mesh(&mut self) {
        let mut font = self.ttf_font.clone();
        let gid = font.lookup('a' as u32).unwrap();
        for ch in '!'..'~' {
            font.lookup(ch as u32).unwrap();
        }
        //maintain a pen that holds the value of teh advanced width
        //eg after a lookup its advance_width valuen and add to the pen
        //use the pen for the base of the next character
        let units_per_em = font.head.units_per_em;
        let cell_advance = font.hmtx.metric_for_glyph(gid as u16).advance_width;
        let cell_ascent = font.hhea.ascent;
        let cell_descent = font.hhea.descent;
        let cell_height = cell_ascent - cell_descent + font.hhea.line_gap;
        let font_size_px = 80;
        let scale = font_size_px as f32 / units_per_em as f32;
        let cell_width_px = cell_advance as f32 * scale;
        let cell_height_px = cell_height as f32 * scale;
        let baseline_offset_px: f32 = cell_ascent as f32 * scale;
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut index_offset = 0u32;
        let screen_w: f32 = 1920.0;
        let screen_h: f32 = 1080.0;
        let cell_count = (screen_w / cell_width_px).floor() as u32;
        let row_count = (screen_h / cell_height_px).floor() as u32;
        println!("cells: {}, rows: {}", cell_count, row_count);
        for row_idx in 0..row_count - 1 {
            for col_idx in 0..cell_count - 1 {
                let rand_letter = rand_letter();
                let gid = font.lookup(rand_letter as u32).unwrap();
                let glyf = *font
                    .parse_gid(gid as u16)
                    .unwrap()
                    .clone()
                    .unwrap()
                    .get_header();
                let x_cell = col_idx as f32 * cell_width_px;
                let y_cell = row_idx as f32 * cell_height_px;
                let baseline_x = x_cell;
                let baseline_y = y_cell + baseline_offset_px;
                let x0 = x_ndc(baseline_x + glyf.x_min as f32 * scale, screen_w);
                let y0 = y_ndc(baseline_y + glyf.y_max as f32 * scale, screen_h);
                let x1 = x_ndc(baseline_x + glyf.x_max as f32 * scale, screen_w);
                let y1 = y_ndc(baseline_y + glyf.y_min as f32 * scale, screen_h);

                if y1 < -1.0 || y0 > 1.0 || y1 > 1.0 || y0 < -1.0 {
                    println!("improper vertex found");
                }

                let ([u0, v0], [u1, v1]) = match Some(rand_letter) {
                    None => ([0.0, 0.0], [0.0, 0.0]),
                    Some(char) => self.atlas.get_uv(char),
                };
                // println!("x0: {}, y0: {}, x1: {}, y1: {}", x0, y0, x1, y1);
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
#[inline(always)]
fn x_ndc(x: f32, screen_w: f32) -> f32 {
    (x / screen_w) * 2.0 - 1.0
}
#[inline(always)]
fn y_ndc(y: f32, screen_h: f32) -> f32 {
    1.0 - (y / screen_h) * 2.0
}
#[derive(Clone)]
pub struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

#[inline(always)]
fn rand_letter() -> char {
    let mut rng = rand::rng();
    let letter_u8 = rng.random_range(b'!'..b'~');
    letter_u8 as char
}
