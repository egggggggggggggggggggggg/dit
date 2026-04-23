use std::{
    borrow::Cow,
    fmt::{Debug, Display},
    path::Path,
    time::Instant,
};

use dit::{
    app::Application,
    dsa::search::{SimilarityCache, TextBuf},
    font_manager::yank_files,
};
use winit::event_loop::{ControlFlow, EventLoop};
fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = Application::new();
    event_loop.run_app(&mut app).unwrap();
    // let file_finder = FileFinder::new();
    // //Input must be sanitized according to the same file sanitazation algo
    // let current = Instant::now();
    // let segs = vec!["jet", "brains", "mono", "nerd", "font", "regular"]
    //     .into_iter()
    //     .map(String::from)
    //     .collect::<Vec<_>>();
    // let tb = TextBuf::new(segs);
    // let mut best_match = "".to_string();
    // let mut cost = f32::MIN;
    // //For this we can use a prie instead of a simple hashmap. This allows for a simple traversal.
    // //Only issue is how to properly build the tree in the first place. Too many allowances and it
    // //takes up too much memory.
    // let mut cache = SimilarityCache::new();
    // let discoverd_files = yank_files("/usr/share/fonts/").unwrap();
    // for file_info in discoverd_files {
    //     let tokens = file_info.tokens;
    //     let mut text_buf = TextBuf::new(tokens);
    //     let res = text_buf.jaccard(&tb, 0.7, &mut cache);
    //     if res > cost {
    //         println!("Name for the better candidate: {:?}", file_info.name);
    //         best_match = file_info.name;
    //         cost = res;
    //     }
    // }
    // //Maybe add a cost threshold here that if at the end of the run is too low disqualifies the
    // //current font to use and instead defaults to a known discoverable font. (sensible font or just
    // //ship one for defaulting to). Could force a dependecy for a specific font type??
    // println!("best match was : {} at a cost of : {}", best_match, cost);
    // println!("Runtime was : {} micros", current.elapsed().as_micros());
}
