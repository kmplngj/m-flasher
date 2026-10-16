//! Finite, length-bounded local helper I/O. No device APIs or generic commands.
//! Deadlines reject late results; they do not preempt a blocked OS filesystem call.

use std::fs::File;
use std::io::{Read, Write};
use std::os::fd::{AsRawFd, FromRawFd, RawFd};
use std::time::{Duration, Instant};

pub struct Pipe {
    input: File,
    output: File,
    pub started: Instant,
    budget: Duration,
}

fn nonblocking(file: &File) -> Result<(), String> {
    // SAFETY: fcntl only reads/changes flags on this owned local IPC descriptor.
    unsafe {
        let flags = libc::fcntl(file.as_raw_fd(), libc::F_GETFL);
        if flags < 0 || libc::fcntl(file.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) < 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
    }
    Ok(())
}

impl Pipe {
    pub fn duplex(file: File) -> Result<Self, String> {
        let output = file.try_clone().map_err(|e| e.to_string())?;
        Self::new(file, output)
    }

    pub fn stdio() -> Result<Self, String> {
        fn duplicate(fd: RawFd) -> Result<File, String> {
            // SAFETY: dup yields a new owned descriptor, checked before ownership.
            let fd = unsafe { libc::dup(fd) };
            if fd < 0 {
                return Err(std::io::Error::last_os_error().to_string());
            }
            Ok(unsafe { File::from_raw_fd(fd) })
        }
        Self::new(duplicate(0)?, duplicate(1)?)
    }

    fn new(input: File, output: File) -> Result<Self, String> {
        nonblocking(&input)?;
        nonblocking(&output)?;
        Ok(Self {
            input,
            output,
            started: Instant::now(),
            budget: Duration::from_secs(900),
        })
    }

    pub fn input_fd(&self) -> RawFd {
        self.input.as_raw_fd()
    }

    fn wait(&self, fd: RawFd, events: i16) -> Result<(), String> {
        loop {
            let left = self
                .budget
                .checked_sub(self.started.elapsed())
                .filter(|d| !d.is_zero())
                .ok_or("FMR1 backup helper deadline expired")?;
            let millis = left.as_millis().saturating_add(1).min(i32::MAX as u128) as i32;
            let mut pollfd = libc::pollfd {
                fd,
                events,
                revents: 0,
            };
            // SAFETY: one valid pollfd for an owned local pipe, finite timeout.
            let result = unsafe { libc::poll(&mut pollfd, 1, millis) };
            if result < 0 {
                let error = std::io::Error::last_os_error();
                if error.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(error.to_string());
            }
            if self.started.elapsed() >= self.budget {
                return Err("FMR1 backup helper deadline expired".into());
            }
            if result == 0 {
                continue;
            }
            // A readable final frame may coexist with HUP; consume it first.
            if pollfd.revents & events != 0 {
                return Ok(());
            }
            return Err("FMR1 backup helper pipe closed".into());
        }
    }

    pub fn read_exact(&mut self, mut data: &mut [u8]) -> Result<(), String> {
        while !data.is_empty() {
            self.wait(self.input.as_raw_fd(), libc::POLLIN)?;
            match self.input.read(data) {
                Ok(0) => return Err("FMR1 backup helper ended mid-frame".into()),
                Ok(n) => {
                    data = &mut data[n..];
                }
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
                    ) => {}
                Err(e) => return Err(e.to_string()),
            }
        }
        if self.started.elapsed() >= self.budget {
            return Err("FMR1 backup helper deadline expired".into());
        }
        Ok(())
    }

    pub fn write_all(&mut self, mut data: &[u8]) -> Result<(), String> {
        while !data.is_empty() {
            self.wait(self.output.as_raw_fd(), libc::POLLOUT)?;
            match self.output.write(data) {
                Ok(0) => return Err("FMR1 backup helper short write".into()),
                Ok(n) => {
                    data = &data[n..];
                }
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
                    ) => {}
                Err(e) => return Err(e.to_string()),
            }
        }
        if self.started.elapsed() >= self.budget {
            return Err("FMR1 backup helper deadline expired".into());
        }
        Ok(())
    }

    pub fn write_frame(&mut self, status: u8, payload: &[u8], max: usize) -> Result<(), String> {
        if payload.len() > max {
            return Err("FMR1 backup helper frame exceeds bound".into());
        }
        self.write_all(&[status])?;
        self.write_all(&(payload.len() as u32).to_le_bytes())?;
        self.write_all(payload)
    }

    pub fn read_frame(&mut self, max: usize) -> Result<(u8, Vec<u8>), String> {
        let mut header = [0u8; 5];
        self.read_exact(&mut header)?;
        let len = u32::from_le_bytes(header[1..].try_into().unwrap()) as usize;
        if len > max {
            return Err("FMR1 backup helper frame exceeds bound".into());
        }
        let mut payload = vec![0; len];
        self.read_exact(&mut payload)?;
        Ok((header[0], payload))
    }
}

pub fn disconnected(fd: RawFd) -> bool {
    let mut p = libc::pollfd {
        fd,
        events: libc::POLLIN,
        revents: 0,
    };
    // SAFETY: nonblocking metadata check on the retained local IPC descriptor.
    let result = unsafe { libc::poll(&mut p, 1, 0) };
    result < 0 || p.revents & (libc::POLLHUP | libc::POLLERR | libc::POLLNVAL | libc::POLLIN) != 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::fd::OwnedFd;
    use std::os::unix::net::UnixStream;

    fn pair() -> (Pipe, Pipe) {
        let (a, b) = UnixStream::pair().unwrap();
        (
            Pipe::duplex(File::from(OwnedFd::from(a))).unwrap(),
            Pipe::duplex(File::from(OwnedFd::from(b))).unwrap(),
        )
    }

    #[test]
    fn bounded_frame_and_closed_peer() {
        let (mut a, mut b) = pair();
        a.write_frame(10, b"local", 5).unwrap();
        assert_eq!(b.read_frame(5).unwrap(), (10, b"local".to_vec()));
        assert!(!disconnected(b.input_fd()));
        drop(a);
        assert!(disconnected(b.input_fd()));
        assert!(b.read_frame(5).is_err());
    }

    #[test]
    fn rejects_oversize_before_allocation_and_expired_deadline() {
        let (mut a, mut b) = pair();
        a.write_all(&[10, 255, 255, 255, 255]).unwrap();
        assert!(b.read_frame(5).unwrap_err().contains("exceeds bound"));
        b.budget = Duration::ZERO;
        assert!(b.read_exact(&mut [0]).unwrap_err().contains("deadline"));
    }
}
