use crossbeam::{channel, epoch::Owned};
use nix::{
    fcntl::OFlag,
    libc::{STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO, TIOCSCTTY, dup2, getpid, setsid},
    poll::{PollFd, PollFlags, PollTimeout},
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
    pub shell: &'static str,
    pub marker: &'static str,
}
impl Pty {
    // Adjust this to take a shell executable path
    // Ex: zsh, fish, bash, sh

    pub fn attempt_create(marker: &'static str) -> nix::Result<Self> {
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

                Ok(Self {
                    master,
                    shell: "/bin/bash",
                    marker,
                })
            }
        }
    }
    /// Polls for input from the slave side of the pty
    /// Thin wrapper around it just handles converting the
    pub fn poll(&self, timeout_ms: i32) -> nix::Result<bool> {
        let mut fds = [PollFd::new(self.master.as_fd(), PollFlags::POLLIN)];
        // If timeout is invalid it just ends itself so should prob add some way of safely panicking
        let ready = nix::poll::poll(&mut fds, PollTimeout::try_from(timeout_ms).unwrap())?;
        if ready == 0 {
            return Ok(false); // timeout
        }

        let revents = fds[0].revents().unwrap_or(PollFlags::empty());

        if revents.contains(PollFlags::POLLERR | PollFlags::POLLHUP) {
            return Err(nix::errno::Errno::EIO);
        }

        Ok(revents.contains(PollFlags::POLLIN))
    }
    /// This function assumes the user has not included
    pub fn write(&mut self, cmds: Vec<&'static str>) -> std::io::Result<()> {
        for cmd in cmds {
            let marker = self.marker;
            let payload = format!("{cmd}\nprintf '{marker} %d\\n' \"$?\"\n",);
            self.master.write_all(payload.as_bytes())?;
        }
        Ok(())
    }
    /// user supplies their own buffer for reading the data into
    /// If the buffer isn't big enough to read data into it returns a message indicating that
    /// via a tuple
    // Issues :
    // How much hard coding should be done for the read function
    // A method is needed to know when the
    pub fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.master.read(buf)?;
        // returns back to the
        Ok(n)
    }
}

#[inline(always)]
pub fn contains_marker(buf: &mut [u8], marker: &[u8]) -> bool {
    buf.ends_with(marker)
}
