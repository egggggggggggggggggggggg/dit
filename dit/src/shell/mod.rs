use crossbeam::channel;
use nix::{
    fcntl::OFlag,
    libc::{STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO, dup2, setsid},
    pty::{PtyMaster, Winsize, grantpt, openpty, posix_openpt, ptsname, unlockpt},
    sys::{stat::Mode, wait::waitpid},
    unistd::{ForkResult, close, execvp, fork},
};
use std::{
    ffi::CString,
    fs::File,
    io::{Read, Write},
    os::fd::{AsRawFd, FromRawFd, IntoRawFd, OwnedFd},
    path::Path,
};
// this should run on another thread

pub fn example() {
    let (tx, rx) = channel::bounded(10);
    std::thread::spawn(move || {
        loop {
            tx.send(0u8);
        }
    });
    if let Ok(data) = rx.try_recv() {

        // The ideal solution for thread communication here is most llikely diff messaging
        // The worker thread sends diffs essentially telling the main thread how to update it's state
        // because its diffs the main thread doesnt have to re update every single thing and instead
        // only focuses on a single item
    }
}

pub fn create_pty2() -> nix::Result<()> {
    let master_fd = posix_openpt(OFlag::O_RDWR)?;
    grantpt(&master_fd)?;
    unlockpt(&master_fd)?;
    let slave_name = unsafe { ptsname(&master_fd) }?;
    let slave_fd = nix::fcntl::open(Path::new(&slave_name), OFlag::O_RDWR, Mode::empty())?;
    match unsafe { fork()? } {
        ForkResult::Child => unsafe {
            setsid();
            // Connects the slave to the standard streams of the slave_fd
            dup2(slave_fd.as_raw_fd(), STDIN_FILENO);
            dup2(slave_fd.as_raw_fd(), STDOUT_FILENO);
            dup2(slave_fd.as_raw_fd(), STDERR_FILENO);
            // The child should only own the slave and thus the master_fd is useless to it.
            close(master_fd)?;
            if slave_fd.as_raw_fd() > 2 {
                // If slave_Fd is not one of the standard streams close it as it would be redundant
                close(slave_fd)?;
            }
            let shell = CString::new("/bin/bash").unwrap();
            execvp(&shell, &[shell.clone()])?;
        },
        ForkResult::Parent { child } => {}
    }
    Ok(())
}

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
