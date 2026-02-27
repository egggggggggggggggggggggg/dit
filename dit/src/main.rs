use std::time::Duration;

use dit::{
    ansii::Parser,
    renderer::App,
    shell::{MarkerMatcher, Pty},
};
use nix::unistd::getpid;
use winit::{
    application::ApplicationHandler,
    event_loop::{ControlFlow, EventLoop},
};
fn main() -> Result<(), std::io::Error> {
    // let event_loop = EventLoop::new().unwrap();
    // event_loop.set_control_flow(ControlFlow::Poll);
    // let mut app = App::new(30, 96);
    // app.generate_screen_mesh();
    // event_loop.run_app(&mut app).unwrap();
    // let master_pty = create_pty().unwrap();
    // let mut parser = Parser::new();
    // parser.advance(0x1b);
    // parser.advance(0x5b);
    let marker = "__DONE__";
    let mut marker_matcher = MarkerMatcher::new(marker.as_bytes());
    let mut pty = Pty::attempt_create(marker).unwrap();
    let cmds = vec!["date", "date", "date", "date", "date", "date"];
    pty.write(&cmds)?;
    // currently blocking, in a real world example poll
    //
    for cmd in cmds {
        let mut buf = [0u8; 4096];
        let mut output = Vec::new();
        loop {
            if pty.poll(10)? {
                let n = pty.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                output.extend_from_slice(&buf[..n]);
                if marker_matcher.feed(&buf[..n]) {
                    marker_matcher.reset();
                    break;
                }
            }
        }
        println!("{}", String::from_utf8_lossy(&output));
    }
    Ok(())
}
// Example of how it might look
// Maintain three
struct Application {
    cmd_queue: Vec<&'static str>,
    // maintains a buffer
    input_queue: Vec<&'static str>,
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
            winit::event::WindowEvent::Resized(size) => {}
            winit::event::WindowEvent::CloseRequested => {
                // safely exit the prgoram here
            }
            winit::event::WindowEvent::RedrawRequested => {
                // draw frame
            }
            _ => {
                // dont do anything
            }
        }
    }
    fn device_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        match event {
        }
    }
}

// Must track the input across frames of the eventLoop.
// Since output is async its not as big of  priority for tracking
//
pub fn event_loop() -> std::io::Result<()> {
    let marker = "__DONE__";
    let mut output = Vec::new();

    let mut marker_matcher = MarkerMatcher::new(marker.as_bytes());
    let mut pty = Pty::attempt_create(marker).unwrap();
    let mut buf = [0u8; 4096];
    loop {
        if pty.poll(1)? {
            let n = pty.read(&mut buf)?;
            if n == 0 {
                break;
            }
            output.extend_from_slice(&buf[..n]);
            if marker_matcher.feed(&buf[..n]) {
                marker_matcher.reset();
                break;
            }
        }
        // calls the draw frame
        let input = get_input();

        // Pretend the input method returns a list of cmds thats already prepocesssed and sanitized

        for cmd in input {}
        draw_frame();
    }
    Ok(())
}

pub fn draw_frame() {}

pub fn get_input() -> Vec<&'static str> {
    Vec::new()
}
