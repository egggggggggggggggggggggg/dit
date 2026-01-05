use dit::{atlas::entry, render::App};
use std::time::Instant;
use winit::event_loop::{ControlFlow, EventLoop};
fn main() {
    //     let start = Instant::now();
    //     entry();
    //     println!("{:?}", start.elapsed());
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}
