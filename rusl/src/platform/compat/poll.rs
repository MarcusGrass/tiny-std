use crate::platform::Fd;

// https://man7.org/linux/man-pages/man2/poll.2.html
transparent_bitflags! {
    pub struct PollEvents: i16 {
        const POLLIN = linux_rust_bindings::poll::POLLIN as i16;
        const POLLPRI = linux_rust_bindings::poll::POLLPRI as i16;
        const POLLOUT = linux_rust_bindings::poll::POLLOUT as i16;
        const POLLERR = linux_rust_bindings::poll::POLLERR as i16;
        const POLLHUP = linux_rust_bindings::poll::POLLHUP as i16;
        const POLLNVAL = linux_rust_bindings::poll::POLLNVAL as i16;
        const POLLRDNORM = linux_rust_bindings::poll::POLLRDNORM as i16;
        const POLLRDBAND = linux_rust_bindings::poll::POLLRDBAND as i16;
        const POLLWRNORM = linux_rust_bindings::poll::POLLWRNORM as i16;
        const POLLWRBAND = linux_rust_bindings::poll::POLLWRBAND as i16;
        const POLLMSG = linux_rust_bindings::poll::POLLMSG as i16;
        const POLLRDHUP = linux_rust_bindings::poll::POLLRDHUP as i16;
    }
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct PollFd(linux_rust_bindings::poll::pollfd);

impl PollFd {
    #[inline]
    #[must_use]
    pub fn new(fd: Fd, events: PollEvents) -> Self {
        Self(linux_rust_bindings::poll::pollfd {
            fd,
            events: events.bits(),
            revents: 0,
        })
    }

    #[must_use]
    pub fn received_events(&self) -> PollEvents {
        self.0.revents.into()
    }
}
