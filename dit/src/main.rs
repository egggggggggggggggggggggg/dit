use std::time::{Duration, Instant};

use dit::{
    ansii::{Handler, Parser},
    app::Application,
    shell::{MarkerMatcher, Pty},
};
use nix::unistd::getpid;
use winit::{
    application::ApplicationHandler,
    event_loop::{ControlFlow, EventLoop},
};
fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = Application::new();
    event_loop.run_app(&mut app).unwrap();
}
