use dit::renderer::App;
use winit::event_loop::{ControlFlow, EventLoop};
fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new(30, 96);
    app.generate_screen_mesh();
    event_loop.run_app(&mut app).unwrap();
}
