use crossbeam::{channel, epoch::Owned};
use nix::{
    fcntl::OFlag,
    libc::{STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO, TIOCSCTTY, dup2, getpid, setsid},
    pty::{PtyMaster, Winsize, grantpt, openpty, posix_openpt, ptsname, unlockpt},
    sys::{
        stat::Mode,
        wait::{WaitPidFlag, waitpid},
    },
    unistd::{ForkResult, close, execvp, fork},
};
use std::{
    ffi::CString,
    fs::File,
    io::{Read, Write},
    os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, IntoRawFd, OwnedFd, RawFd},
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
use nix::sys::termios::{LocalFlags, SetArg, tcgetattr, tcsetattr};

fn disable_echo(fd: BorrowedFd) -> nix::Result<()> {
    let mut termios = tcgetattr(fd)?;
    termios
        .local_flags
        .remove(LocalFlags::ECHO | LocalFlags::ICANON);
    tcsetattr(fd, SetArg::TCSANOW, &termios)?;
    Ok(())
}
pub struct Pty {
    pub master: File,
}
impl Pty {
    pub fn attempt_create() -> nix::Result<Self> {
        let master_fd = posix_openpt(OFlag::O_RDWR)?;
        grantpt(&master_fd)?;
        unlockpt(&master_fd)?;
        let slave_name = unsafe { ptsname(&master_fd) }?;
        let slave_fd = nix::fcntl::open(Path::new(&slave_name), OFlag::O_RDWR, Mode::empty())?;
        disable_echo(slave_fd.as_fd())?;
        println!("Forking the process");
        println!("current process id: {}", unsafe { getpid() });
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
                execvp(
                    &shell,
                    &[
                        shell.clone(),
                        CString::new("--noprofile").unwrap(),
                        CString::new("--norc").unwrap(),
                        CString::new("-i").unwrap(),
                    ],
                )?;
                //
                // Tells the compiler that this branch will not ccontinue executing code
                unreachable!();
            },
            ForkResult::Parent { child } => {
                close(slave_fd.as_raw_fd())?;
                let master = unsafe { File::from_raw_fd(master_fd.into_raw_fd()) };
                waitpid(child, Some(WaitPidFlag::WNOHANG))?;
                println!("parent process id: {}", unsafe { getpid() });

                Ok(Self { master })
            }
        }
    }
    pub fn sanitize() {}
    // Synchornization issues as the thing is reading before the slave has actually sent anything back
    // Instead of using some arbirtrary marker set one. 
    pub fn test(&mut self) -> std::io::Result<()> {
        println!("test");
        self.master.write_all(b"PS1='PROMPT_READY> '\n")?;
        self.master.write_all(b"date\n")?;
        println!("finished writing");
        let mut output = Vec::new();
        let mut buf = [0u8; 1024];

        loop {
            let n = self.master.read(&mut buf)?;
            output.extend_from_slice(&buf[..n]);
            if output.ends_with(b"$ ") {
                break;
            } // crude prompt detection
        }

        println!("ls output:\n{}", String::from_utf8_lossy(&output));
        Ok(())
    }
}

pub fn create_pty2() -> nix::Result<File> {
    let master_fd = posix_openpt(OFlag::O_RDWR)?;
    grantpt(&master_fd)?;
    unlockpt(&master_fd)?;
    let slave_name = unsafe { ptsname(&master_fd) }?;
    let slave_fd = nix::fcntl::open(Path::new(&slave_name), OFlag::O_RDWR, Mode::empty())?;
    match unsafe { fork()? } {
        ForkResult::Child => unsafe {
            // Since this is now in an entirely new process, can;t see output from it
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
            // Tells the compiler that this branch will not ccontinue executing code
            unreachable!();
        },
        ForkResult::Parent { child } => {
            close(slave_fd.as_raw_fd())?;
            let mut master = unsafe { File::from_raw_fd(master_fd.as_raw_fd()) };
            let mut buffer = [0u8; 1024];
            waitpid(child, None)?;
            Ok(master)
        }
    }
    // Once the forking has been completed store the slave and master
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
        // When forking a new processes gets spawned, thats the child, the child executes this
        // branch of the match statement
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
        // The parent executes this one. In essence because theres multiple threads both branches get executed.
        ForkResult::Parent { child } => {
            close(pty.slave.as_raw_fd())?;
            let mut master = unsafe { File::from_raw_fd(pty.master.as_raw_fd()) };
            let mut buffer = [0u8; 1024];
            waitpid(child, None)?;
        }
    }
    Ok(pty.master)
}
