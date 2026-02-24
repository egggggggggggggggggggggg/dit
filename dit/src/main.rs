use dit::{ansii::Parser, renderer::App, shell::create_pty};
use winit::event_loop::{ControlFlow, EventLoop};
fn main() {
    // let event_loop = EventLoop::new().unwrap();
    // event_loop.set_control_flow(ControlFlow::Poll);
    // let mut app = App::new(30, 96);
    // app.generate_screen_mesh();
    // event_loop.run_app(&mut app).unwrap();
    // let master_pty = create_pty().unwrap();
    let mut parser = Parser::new();
    parser.advance(0x1b);
    parser.advance(0x5b);
    
}
