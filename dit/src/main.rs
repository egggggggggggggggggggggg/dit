use std::time::Duration;

use dit::{
    ansii::Parser,
    renderer::App,
    shell::{Pty, contains_marker},
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
    let mut pty = Pty::attempt_create(marker).unwrap();
    pty.write(vec!["ls", "date", "printf a"])?;
    // currently blocking, in a real world example poll
    // would get called in an event loop with no poll condition blocking
    let mut buf = [0u8; 4096];
    let mut output = Vec::new();
    let test_lim = 1000;
    let mut counter = 0;
    loop {
        if pty.poll(0)? {
            println!("there is data to read");
            let n = pty.read(&mut buf)?;
            output.extend_from_slice(&buf[..n]);
            if contains_marker(&mut buf, marker.as_bytes()) {
                break;
            }
        } else {
            if counter >= test_lim {
                break;
            }
            println!("there is no data to read");
        }
        counter += 1;
    }

    println!("{}", String::from_utf8_lossy(&output));
    Ok(())
}
