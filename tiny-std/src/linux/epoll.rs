use rusl::platform::EpollOp;
pub use rusl::platform::{EpollEvent, EpollEventMask};

use crate::error::{Error, Result};
use crate::unix::fd::{OwnedFd, RawFd};

pub struct EpollDriver {
    epoll_fd: OwnedFd,
}

#[derive(Debug, Copy, Clone)]
pub enum EpollTimeout {
    WaitForever,
    WaitMillis(u32),
    NoWait,
}

impl EpollDriver {
    /// Creates a new epoll driver
    /// # Errors
    /// OS errors during setup syscall
    #[inline]
    pub fn create(cloexec: bool) -> Result<Self> {
        let fd = rusl::select::epoll_create(cloexec)?;
        Ok(Self {
            epoll_fd: OwnedFd(fd),
        })
    }

    /// Registers a new entry into the `EpollDriver`'s underlying kernel state.
    /// The identifier is any u64 which the user can use to tell ready fds apart or indeed
    /// store any data they'd like.
    /// # Errors
    /// OS errors registering the fd, such as a nonsensical mask, or a bad provided `fd`
    #[inline]
    pub fn register(&self, fd: RawFd, identifier: u64, mask: EpollEventMask) -> Result<()> {
        rusl::select::epoll_ctl(
            self.epoll_fd.0,
            EpollOp::Add,
            fd,
            &EpollEvent::new(identifier, mask),
        )?;
        Ok(())
    }

    /// Unregisters an `fd` from the `EpollDriver`'s underlying kernel state.
    /// # Errors
    /// OS errors unregistering the fd, such as a bad provided `fd`
    #[inline]
    pub fn unregister(&self, fd: RawFd) -> Result<()> {
        rusl::select::epoll_del(self.epoll_fd.0, fd)?;
        Ok(())
    }

    /// Replaces the previous registration for the provided fd with a new one.
    /// # Errors
    /// OS errors modifying the fd, such as a bad provided `fd` or a nonsensical `mask`.
    #[inline]
    pub fn modify(&self, fd: RawFd, identifier: u64, mask: EpollEventMask) -> Result<()> {
        rusl::select::epoll_ctl(
            self.epoll_fd.0,
            EpollOp::Mod,
            fd,
            &EpollEvent::new(identifier, mask),
        )?;
        Ok(())
    }

    /// Waits for any registered `fd` to become ready.
    /// The `timeout` can at most be `i32::MAX` milliseconds
    /// # Errors
    /// Os errors occurring during wait, or a timeout that is too long
    #[inline]
    #[allow(clippy::cast_possible_wrap)]
    pub fn wait(&self, event_buf: &mut [EpollEvent], timeout: EpollTimeout) -> Result<usize> {
        let num_ready = match timeout {
            EpollTimeout::WaitForever => rusl::select::epoll_wait(self.epoll_fd.0, event_buf, -1)?,
            EpollTimeout::WaitMillis(time) => {
                if time > i32::MAX as u32 {
                    return Err(Error::Uncategorized(
                        "Epoll wait with a timeout bigger than i32::MAX",
                    ));
                }
                rusl::select::epoll_wait(self.epoll_fd.0, event_buf, time as i32)?
            }
            EpollTimeout::NoWait => rusl::select::epoll_wait(self.epoll_fd.0, event_buf, 0)?,
        };
        Ok(num_ready)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusl::platform::STDIN;

    #[test]
    fn test_epoll_driver() {
        let drive = EpollDriver::create(true).unwrap();
        drive.register(STDIN, 1, EpollEventMask::EPOLLOUT).unwrap();
        let mut buf = [EpollEvent::new(0, EpollEventMask::empty())];
        drive
            .wait(&mut buf, EpollTimeout::WaitMillis(1_000))
            .unwrap();
        assert_eq!(1, buf[0].get_data());
        assert!(buf[0].get_events().contains(EpollEventMask::EPOLLOUT), "Expected EPOLLOUT, got {:?}", buf[0].get_events());
    }
}
