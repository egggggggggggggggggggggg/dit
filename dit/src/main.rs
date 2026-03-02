use std::time::{Duration, Instant};

use dit::{
    ansii::{Handler, Parser},
    renderer::App,
    shell::{MarkerMatcher, Pty},
};
use nix::unistd::getpid;
use winit::{
    application::ApplicationHandler,
    event_loop::{ControlFlow, EventLoop},
};
fn main() -> Result<(), std::io::Error> {
    // let event_loop = EventLoop::new().unwrap();
    // event_loop.set_control_flow(ControlFlow::Poll);
    // let mut app = App::new(30, 96);
    // app.generate_screen_mesh();
    // event_loop.run_app(&mut app).unwrap();
    // let sequences = [
    //     // SGR: colors and styles
    //     "\x1b[0m",            // reset
    //     "\x1b[1m",            // bold
    //     "\x1b[3m",            // italic
    //     "\x1b[4m",            // underline
    //     "\x1b[7m",            // reverse
    //     "\x1b[30m",           // black text
    //     "\x1b[31m",           // red text
    //     "\x1b[32m",           // green text
    //     "\x1b[33m",           // yellow text
    //     "\x1b[34m",           // blue text
    //     "\x1b[35m",           // magenta text
    //     "\x1b[36m",           // cyan text
    //     "\x1b[37m",           // white text
    //     "\x1b[1;31m",         // bold red
    //     "\x1b[0;33;44m",      // yellow on blue
    //     "\x1b[38;5;196m",     // 256-color red
    //     "\x1b[48;5;46m",      // 256-color background green
    //     "\x1b[38;2;255;0;0m", // truecolor red
    //     "\x1b[48;2;0;255;0m", // truecolor background green
    //     // Cursor movement
    //     "\x1b[H",      // home
    //     "\x1b[10;20H", // row 10, col 20
    //     "\x1b[5A",     // cursor up 5
    //     "\x1b[3B",     // cursor down 3
    //     "\x1b[2C",     // cursor forward 2
    //     "\x1b[4D",     // cursor back 4
    //     // Erase/clear
    //     "\x1b[2J", // clear screen
    //     "\x1b[K",  // clear line from cursor to end
    //     "\x1b[1K", // clear line from start to cursor
    //     "\x1b[2K", // clear entire line
    //     // Save/restore cursor
    //     "\x1b[s", // save cursor
    //     "\x1b[u", // restore cursor
    //     // Hide/show cursor
    //     "\x1b[?25l", // hide
    //     "\x1b[?25h", // show
    //     // OSC sequences
    //     "\x1b]0;Title\x07",       // set window title
    //     "\x1b]52;c;SGVsbG8=\x07", // clipboard copy
    //     // Random/invalid sequences (should not crash parser)
    //     "\x1b[999;999m",
    //     "\x1b[?999h",
    //     "\x1b[?999l",
    //     "\x1b[1;2;3;4;5;6;7;8;9;10m",
    //     "\x1b[;m",    // empty param
    //     "\x1b[1;;3m", // double semicolon
    //     "\x1b[999m",  // high invalid param
    //     "plain text", // non-escape
    // ];
    // let mut parser = Parser::new();
    // for sequence in sequences {
    //     for byte in sequence.as_bytes() {}
    // }
    unsafe { std::env::set_var("RUST_BACKTRACE", "full") };
    let marker = "__DONE__";
    let mut marker_matcher = MarkerMatcher::new(marker.as_bytes());
    let ws = libc::winsize {
        ws_row: 24,
        ws_col: 80,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    let mut pty = Pty::attempt_create(marker, ws)?;
    let cmds = vec!["date".to_string(), "".to_string()];

    pty.write(&cmds)?;
    // currently blocking, in a real world example poll
    //
    //
    let mut instant;
    for _ in cmds {
        instant = Instant::now();
        let mut buf = [0u8; 128];
        let mut output = Vec::new();
        loop {
            if pty.poll(10)? || instant.elapsed().as_millis() >= 50 {
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
            println!("{}", String::from_utf8_lossy(&output));
            buf = [0u8; 128];
        }
    }
    Ok(())
}
