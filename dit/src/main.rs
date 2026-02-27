use std::time::Duration;

use dit::{
    ansii::Parser,
    renderer::App,
    shell::{MarkerMatcher, Pty},
};
use nix::unistd::getpid;
use winit::event_loop::{ControlFlow, EventLoop};
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
