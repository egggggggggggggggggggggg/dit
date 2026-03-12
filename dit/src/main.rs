use std::{borrow::Cow, path::Path};

use dit::{
    app::Application,
    font_manager::{
        FileFinder,
        search::{Words, jaccard, lcs, levenshtein_distance},
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
    // let seg = &"JetBrainsMononerdFontBold".to_lowercase();
    // let mut best_match = "".to_string();
    // let mut cost = usize::max_value();
    // let discoverd_files = FileFinder::yank_files("/usr/share/fonts/").unwrap();
    // for file in discoverd_files {
    //     let res = levenshtein_distance(seg, &file);
    //     if res < cost {
    //         println!("res: {}, file: {}", res, file);
    //         cost = res;
    //         best_match = file;
    //     }
    // }
    // println!("best match was : {} at a cost of : {}", best_match, cost);
    // let test_1 = "fteaiejmf";
    // let test_2 = "facetimefacetimefacetime";
    // let val = lcs(test_1, test_2);
    // println!("val: {}", val);
    let words1: Words = vec![
        Cow::Borrowed("aple"),
        Cow::Borrowed("dog"),
        Cow::Borrowed("egg"),
    ];
    let words2: Words = vec![
        Cow::Borrowed("apple"),
        Cow::Borrowed("aple"),
        Cow::Borrowed("beg"),
    ];
    let sim = jaccard(&words1, &words2);
    let threshold = 0.7;
    println!("Jaccard similarity: {}", sim);
}
