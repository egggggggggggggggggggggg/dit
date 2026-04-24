use std::{collections::HashSet, time::Instant};

use atlas_gen::{
    allocator::ShelfAllocator, atlas::Atlas, cont_comb::SimpleContourCombiner,
    edge_coloring::edge_coloring_simple, edge_select::MultiDistanceSelector,
    shape_distance_finder::ShapeDistanceFinder,
};
use font_parser::TtfFont;
use image::{ImageBuffer, Rgb};
use libc::winsize;
use math::lalg::Vec2;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalSize},
    event::{DeviceEvent, ElementState, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use crate::{
    ansii::Parser,
    renderer::{shader::Vertex, vkapp::VkApplication},
    screen::Screen,
    shell::Pty,
};
// In seconds

pub struct Application {
    screen: Option<Screen>,
    pressed_keys: HashSet<KeyCode>,
    input_buffer: String,
    window: Option<Window>,
    last_frame: Instant,
    last_blink: Instant,
    pty: Pty,
    parser: Parser,
    vk_app: Option<VkApplication>,
    frame_count: u32,
}
const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;

impl Application {
    fn update(&mut self) {
        self.frame_count += 1;
        let screen = self.screen.as_mut().unwrap();
        if self.last_blink.elapsed().as_secs_f64() <= 0.5 {
            screen.cursor.visible = true;
        }
        // only write if the input buffer is not empty
        if !self.input_buffer.is_empty() {
            self.pty.write(&self.input_buffer).unwrap();
            for char in self.input_buffer.chars() {
                screen.write_char(char);
            }
            self.input_buffer.clear();
        }
        if self.pty.poll(0).unwrap() {
            // Checks for data to read from
            let mut buf = [0u8; 4096];
            // Staging buffer is created to handle the bytes
            let n = self.pty.read(&mut buf).unwrap();
            // Read writes into said buffer
            if n != 0 {
                println!("recieved input: {}", String::from_utf8_lossy(&buf));
                // Checks for if there are bytes to consume
                for byte in &buf[..n] {
                    self.parser.consume(*byte, screen);
                }
            }
        }
        // at the end of the poll check if the mesh needs to be reupdated
        // if yes do so
        let vertex_size = size_of::<Vertex>();
        if let Some(mut diffs) = screen.update_mesh() {
            //convert the diffs to vk::CopyBuffers
            let mut regions = Vec::new();
            let vk_app = self.vk_app.as_mut().unwrap();
            for diff in &mut diffs {
                vk_app.vertex_buffer.write_into_staging::<u32, _>(
                    &screen.mesh.vertices[diff.start..diff.end],
                    (diff.start * vertex_size) as u64,
                );
                // Range/diff operates on just the struct and not teh actual count of bytes
                diff.start = diff.start * vertex_size;
                diff.end = diff.end * vertex_size;
                let region = ash::vk::BufferCopy::from(diff.clone());
                regions.push(region);
            }
            vk_app.write_to_device(&regions);
        }
    }
    pub fn new() -> Self {
        Self {
            screen: None,
            // arbitrary pre allocated space
            input_buffer: String::with_capacity(4096),
            window: None,
            last_frame: Instant::now(),
            last_blink: Instant::now(),
            pty: Pty::attempt_create(
                "__PLACEHOLDER__",
                winsize {
                    ws_row: 0,
                    ws_col: 0,
                    ws_xpixel: 0,
                    ws_ypixel: 0,
                },
            )
            .unwrap(),
            parser: Parser::new(),
            vk_app: None,
            pressed_keys: HashSet::new(),
            frame_count: 0,
        }
    }
}
// this is a temporary solution just to ensure it works first
// will change it later
const CROSS_THRESHOLD: f64 = 3.0;
fn preload_latin(font: &mut TtfFont, texture_atlas: &mut Atlas<char, Rgb<u8>, ShelfAllocator>) {
    let target_font_px = 32;
    let dmax_px = 1.0;
    for ch in '!'..'~' {
        let mut seed = 0;
        let gid = font.lookup(ch as u32).unwrap();
        let mut shape = font.assemble_glyf(gid as u16).unwrap();
        // shape.normalize();
        edge_coloring_simple(&mut shape, CROSS_THRESHOLD.sin(), &mut seed);
        let glyph = font.glyf.get_glyf(gid as u16).unwrap().clone();
        let bounds = glyph.get_header();
        let mut sdf: ShapeDistanceFinder<SimpleContourCombiner<MultiDistanceSelector>> =
            ShapeDistanceFinder::new(shape);
        // scale = pixels per font unit
        let scale = target_font_px as f64 / font.head.units_per_em as f64;

        // convert dMAX from pixels to font units
        let dmax = dmax_px / scale;
        let distance_range = 2.0 * dmax;
        let max_color = 255.0;

        let width = bounds.x_max - bounds.x_min;
        let height = bounds.y_max - bounds.y_min;

        let pixel_width = (width as f64 * scale).ceil().max(1.0) as u32;
        let pixel_height = (height as f64 * scale).ceil().max(1.0) as u32;

        let mut output_image: ImageBuffer<Rgb<u8>, Vec<u8>> =
            ImageBuffer::new(pixel_width, pixel_height);

        for py in 0..pixel_height {
            for px in 0..pixel_width {
                let gx = bounds.x_min as f64 + (px as f64 + 0.5) / scale;
                let gy = bounds.y_min as f64 + (py as f64 + 0.5) / scale;
                let p = Vec2 { x: gx, y: gy };
                let distance = sdf.distance(p);

                // clamp to [-dmax, +dmax]
                let clamped_r = distance.r.clamp(-dmax, dmax);
                let clamped_g = distance.g.clamp(-dmax, dmax);
                let clamped_b = distance.b.clamp(-dmax, dmax);

                // distanceColor(d) = ((d / (2*dmax)) + 0.5) * 255
                let r_0_255 = ((clamped_r / distance_range + 0.5) * max_color)
                    .clamp(0.0, 255.0)
                    .round() as u8;

                let g_0_255 = ((clamped_g / distance_range + 0.5) * max_color)
                    .clamp(0.0, 255.0)
                    .round() as u8;

                let b_0_255 = ((clamped_b / distance_range + 0.5) * max_color)
                    .clamp(0.0, 255.0)
                    .round() as u8;
                let pixel = Rgb([r_0_255, g_0_255, b_0_255]);
                output_image.put_pixel(px, pixel_height - 1 - py, pixel);
            }
        }
        texture_atlas.add_image(ch, &output_image).unwrap();
        // output_image.save(format!("./res/{}.png", ch)).unwrap();
    }

    texture_atlas
        .image
        .save("./assets/texture_atlas.png")
        .unwrap();
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window_size = PhysicalSize::new(1920, 1080);
        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_title("dit")
                    .with_inner_size(window_size),
            )
            .unwrap();
        // currently the values are hardcoded. ill add some sort of way of configuring these settings instead of baking it in
        let mut font = TtfFont::new("../JetBrainsMonoNerdFontMono-Regular.ttf").unwrap();
        let atlas_allocator = ShelfAllocator::new(512, 512);
        let mut texture_atlas: Atlas<char, Rgb<u8>, ShelfAllocator> =
            Atlas::new(1024, 1024, atlas_allocator, 4);
        preload_latin(&mut font, &mut texture_atlas);
        let mut screen = Screen::new(
            12.0,
            font,
            texture_atlas,
            LogicalSize::from_physical(window_size, window.scale_factor()),
        );
        screen.construct_mesh();
        self.vk_app = Some(VkApplication::new(&window, &screen.mesh));
        self.window = Some(window);
        self.last_frame = Instant::now();
        self.last_blink = Instant::now();
        self.screen = Some(screen);
    }
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::RedrawRequested => {
                // let now = Instant::now();
                // let dt = now - self.last_frame;
                // self.last_frame = now;
                self.update();
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
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                let app = self.vk_app.as_mut().unwrap();
                app.resize_dimensions = [new_size.width, new_size.height];
                app.recreate_swapchain();
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
        if self.last_frame.elapsed().as_secs_f32() >= 1.0 {
            self.frame_count = 0;
            self.last_frame = Instant::now();
        }
        self.window.as_mut().unwrap().request_redraw();
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
    fn exiting(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.vk_app.as_ref().unwrap().wait_gpu_idle();
    }
}
