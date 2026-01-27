use atlas_gen::entry;
use dit::renderer::App;
use font_parser::ttf_parse::TtfFont;
use winit::{
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    raw_window_handle::HasDisplayHandle,
    window::WindowAttributes,
};
const FONT_SIZE: u32 = 16;
fn main() {
    //     unsafe {
    //         std::env::set_var("RUST_BACKTRACE", "1");
    //     }

    //load the config files
    //parse it and use those as the default parameters
    //for now use constants in place of the config

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new(27, 48);
    //this shouldnt be some arbitaray number but based off a standard like from the font file or smth else
    
    app.generate_screen_mesh();
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
