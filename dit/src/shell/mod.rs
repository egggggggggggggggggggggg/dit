use std::{
    ffi::CString,
    fs::File,
    io::{Read, Write},
    os::fd::{AsRawFd, FromRawFd, IntoRawFd, OwnedFd},
};

use nix::{
    libc::{STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO, dup2, setsid},
    pty::{Winsize, openpty},
    sys::wait::waitpid,
    unistd::{ForkResult, close, execvp, fork},
};

pub fn create_pty() -> nix::Result<OwnedFd> {
    let winsize = Winsize {
        ws_row: 24,
        ws_col: 80,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    let pty = openpty(Some(&winsize), None)?;
    
    match unsafe { fork()? } {
        ForkResult::Child => unsafe {
            let src = &pty.slave.as_raw_fd();
            setsid();
            dup2(*src, STDIN_FILENO);
            dup2(*src, STDOUT_FILENO);
            dup2(*src, STDERR_FILENO);
            close(pty.slave.as_raw_fd())?;
            close(pty.master.as_raw_fd())?;
            //Bash shell
            let shell = CString::new("/bin/bash").unwrap();
            execvp(&shell, &[shell.clone()])?;
        },
        ForkResult::Parent { child } => {
            close(pty.slave.as_raw_fd())?;
            let mut master = unsafe { File::from_raw_fd(pty.master.as_raw_fd()) };
            let mut buffer = [0u8; 1024];
            waitpid(child, None)?;
        }
    }
    Ok(pty.master)
}
