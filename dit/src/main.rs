use atlas_gen::entry;
use dit::renderer::App;
use font_parser::ttf_parse::TtfFont;
use winit::event_loop::{ControlFlow, EventLoop};
fn main() {
    //     unsafe {
    //         std::env::set_var("RUST_BACKTRACE", "1");
    //     }
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    event_loop.run_app(&mut app).unwrap();
}

fn median<T>(a: T, b: T, c: T) -> T
where
    T: Ord + Copy,
{
    (a.min(b)).max(b.min(c))
}
//refactoring
//currently the entire thing is basically baked in
//change the pipeline to allow passing in of uniforms and whatnot
//setup a simple config file parser
