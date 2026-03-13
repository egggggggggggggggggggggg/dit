use std::{borrow::Cow, path::Path};

use dit::{
    app::Application,
    font_manager::{
        FileFinder,
        search::{TextBuf, jaccard},
    },
};
use winit::event_loop::{ControlFlow, EventLoop};
fn main() {
    //let event_loop = EventLoop::new().unwrap();
    //event_loop.set_control_flow(ControlFlow::Poll);
    // let mut app = Application::new();
    // event_loop.run_app(&mut app).unwrap();
    // let file_finder = FileFinder::new();
    // //Input must be sanitized according to the same file sanitazation algo.
    let segs = ["adobe", "new", "century", "schoolbook"]
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>();
    let tb = TextBuf::new(segs);
    let mut best_match = "".to_string();
    let mut cost = f32::MIN;
    let discoverd_files = FileFinder::yank_files("/usr/share/fonts/").unwrap();
    for file_info in discoverd_files {
        let tokens = file_info.tokens;
        let text_buf = TextBuf::new(tokens);
        let res = jaccard(&tb, &text_buf);
        if res > cost {
            println!("Name for the better candidate: {:?}", file_info.name);
            best_match = file_info.name;
            cost = res;
        }
    }
    println!("best match was : {} at a cost of : {}", best_match, cost);
}
