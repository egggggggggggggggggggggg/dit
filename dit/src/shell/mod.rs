use nix::ioctl_write_ptr;
use nix::sys::termios::{LocalFlags, SetArg, tcgetattr, tcsetattr};
use nix::{
    fcntl::{FcntlArg::F_SETFL, OFlag, fcntl},
    libc::{STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO, dup2, getpid, setsid},
    poll::{PollFd, PollFlags, PollTimeout},
    pty::{grantpt, posix_openpt, ptsname, unlockpt},
    sys::{
        stat::Mode,
        wait::{WaitPidFlag, waitpid},
    },
    unistd::{ForkResult, close, execvp, fork},
};
use std::io::{BufRead, BufReader};
use std::{
    ffi::CString,
    fs::File,
    io::{Read, Write},
    os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, IntoRawFd},
    path::Path,
};

#[inline(always)]
fn disable_echo(fd: BorrowedFd) -> nix::Result<()> {
    let mut termios = tcgetattr(fd)?;
    termios
        .local_flags
        .remove(LocalFlags::ECHO | LocalFlags::ICANON);
    tcsetattr(fd, SetArg::TCSANOW, &termios)?;
    Ok(())
}
/// Tracks marker matching efficiently across streamed input
pub struct MarkerMatcher {
    marker: Vec<u8>,
    matched: usize,
}

impl MarkerMatcher {
    pub fn new(marker: &[u8]) -> Self {
        Self {
            marker: marker.to_vec(),
            matched: 0,
        }
    }

    /// Feed bytes, returns true if the marker is fully matched
    pub fn feed(&mut self, bytes: &[u8]) -> bool {
        for &b in bytes {
            if b == self.marker[self.matched] {
                self.matched += 1;
                if self.matched == self.marker.len() {
                    // Full marker matched
                    self.matched = 0; // reset for next match
                    return true;
                }
            } else {
                // Mismatch: check if current byte starts a partial match
                self.matched = if b == self.marker[0] { 1 } else { 0 };
            }
        }
        false
    }

    pub fn reset(&mut self) {
        self.matched = 0;
    }
}
// Sent to specify the shell and additional parameters
#[inline(always)]
#[cfg(target_os = "linux")]
pub fn available_shells() -> Vec<String> {
    let file = File::open("/etc/shells").unwrap();
    let reader = BufReader::new(file);
    reader
        .lines()
        .filter_map(|line| {
            let line = line.ok()?.trim().to_string();
            if line.starts_with('#') || line.is_empty() {
                None
            } else {
                Some(line)
            }
        })
        .collect()
}
struct ShellConfig {
    shell: String,
    args: Vec<String>,
}

impl ShellConfig {
    fn new(shell: String) -> Self {
        Self {
            shell,
            args: Vec::new(),
        }
    }
}
impl Default for ShellConfig {
    fn default() -> Self {
        if let Ok(shell) = std::env::var("SHELL") {
            println!("User preferred shell: {}", shell);
            Self {
                shell,
                args: Vec::new(),
            }
        } else {
            Self {
                shell: "".to_string(),
                args: Vec::new(),
            }
        }
    }
}
struct PtyConfig {
    shell: ShellConfig,
    marker: String,
}

pub struct Pty {
    pub master: File,
    pub shell: &'static str,
    pub marker: &'static str,
}

impl Pty {
    // Adjust this to take a shell executable path
    // Ex: zsh, fish, bash, sh

    pub fn attempt_create(marker: &'static str, win_size: libc::winsize) -> nix::Result<Self> {
        ioctl_write_ptr!(tiocswinsz, b'T', libc::TIOCSWINSZ, libc::winsize);
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
                // let slave_fd =
                //     nix::fcntl::open(Path::new(&slave_name), OFlag::O_RDWR, Mode::empty())?;
                libc::ioctl(slave_fd.as_raw_fd(), libc::TIOCSCTTY, 0);
                // Connects the slave to the standard streams of the slave_fd
                dup2(slave_fd.as_raw_fd(), STDIN_FILENO);
                dup2(slave_fd.as_raw_fd(), STDOUT_FILENO);
                dup2(slave_fd.as_raw_fd(), STDERR_FILENO);
                // tiocswinsz(slave_fd.as_raw_fd(), &win_size)?;
                // The child should only own the slave and thus the master_fd is useless to it.
                close(master_fd)?;
                if slave_fd.as_raw_fd() > 2 {
                    // If slave_Fd is not one of the standard streams close it as it would be redundant
                    close(slave_fd)?;
                }

                let shell = CString::new("/bin/bash").unwrap();
                let args = [
                    shell.clone(),
                    CString::new("--noprofile").unwrap(),
                    CString::new("--norc").unwrap(),
                    CString::new("-i").unwrap(),
                    CString::new("-c").unwrap(),
                    CString::new("export PS1=''; exec bash").unwrap(),
                ];

                execvp(&shell, &args)?;
                //
                // Tells the compiler that this branch will not ccontinue executing code
                unreachable!();
            },
            ForkResult::Parent { child } => {
                let master = unsafe { File::from_raw_fd(master_fd.into_raw_fd()) };
                waitpid(child, Some(WaitPidFlag::WNOHANG))?;
                println!("parent process id: {}", unsafe { getpid() });
                fcntl(&master, F_SETFL(OFlag::O_NONBLOCK))?;
                Ok(Self {
                    master,
                    shell: "/bin/bash",
                    marker,
                })
            }
        }
    }
    /// Polls for input from the slave side of the pty
    /// Thin wrapper around it just handles checking for POLLIN
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
    pub fn write(&mut self, input: &String) -> std::io::Result<()> {
        println!("writing this string: {:x?}", input.as_bytes());
        self.master.write_all(input.as_bytes())?;
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
