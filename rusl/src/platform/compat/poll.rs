use crate::platform::{comptime_i32_to_i16, Fd};

// https://man7.org/linux/man-pages/man2/poll.2.html
transparent_bitflags! {
    pub struct PollEvents: i16 {
        const DEFAULT = 0;
        const POLLIN = comptime_i32_to_i16(linux_rust_bindings::poll::POLLIN);
        const POLLPRI = comptime_i32_to_i16(linux_rust_bindings::poll::POLLPRI);
        const POLLOUT = comptime_i32_to_i16(linux_rust_bindings::poll::POLLOUT);
        const POLLERR = comptime_i32_to_i16(linux_rust_bindings::poll::POLLERR);
        const POLLHUP = comptime_i32_to_i16(linux_rust_bindings::poll::POLLHUP);
        const POLLNVAL = comptime_i32_to_i16(linux_rust_bindings::poll::POLLNVAL);
        const POLLRDNORM = comptime_i32_to_i16(linux_rust_bindings::poll::POLLRDNORM);
        const POLLRDBAND = comptime_i32_to_i16(linux_rust_bindings::poll::POLLRDBAND);
        const POLLWRNORM = comptime_i32_to_i16(linux_rust_bindings::poll::POLLWRNORM);
        const POLLWRBAND = comptime_i32_to_i16(linux_rust_bindings::poll::POLLWRBAND);
        const POLLMSG = comptime_i32_to_i16(linux_rust_bindings::poll::POLLMSG);
        const POLLRDHUP = comptime_i32_to_i16(linux_rust_bindings::poll::POLLRDHUP);
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
            fd: fd.0,
            events: events.bits(),
            revents: 0,
        })
    }

    #[must_use]
    pub fn received_events(&self) -> PollEvents {
        self.0.revents.into()
    }
}
