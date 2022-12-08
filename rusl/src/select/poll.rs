use sc::syscall;

use crate::platform::signal::SigSetT;
use crate::platform::{Fd, TimeSpec};

// https://man7.org/linux/man-pages/man2/poll.2.html
transparent_bitflags! {
    pub struct PollEvents: i16 {
        const POLLIN = linux_rust_bindings::POLLIN as i16;
        const POLLPRI = linux_rust_bindings::POLLPRI as i16;
        const POLLOUT = linux_rust_bindings::POLLOUT as i16;
        const POLLERR = linux_rust_bindings::POLLERR as i16;
        const POLLHUP = linux_rust_bindings::POLLHUP as i16;
        const POLLNVAL = linux_rust_bindings::POLLNVAL as i16;
        const POLLRDNORM = linux_rust_bindings::POLLRDNORM as i16;
        const POLLRDBAND = linux_rust_bindings::POLLRDBAND as i16;
        const POLLWRNORM = linux_rust_bindings::POLLWRNORM as i16;
        const POLLWRBAND = linux_rust_bindings::POLLWRBAND as i16;
        const POLLMSG = linux_rust_bindings::POLLMSG as i16;
        const POLLRDHUP = linux_rust_bindings::POLLRDHUP as i16;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PollFd {
    fd: i32,
    events: PollEvents,
    revents: PollEvents,
}

impl PollFd {
    #[inline]
    #[must_use]
    pub fn new(fd: Fd, events: PollEvents) -> Self {
        Self {
            fd,
            events,
            revents: PollEvents::empty(),
        }
    }

    #[must_use]
    pub fn received_events(&self) -> PollEvents {
        self.revents
    }
}

/// Polls the provided fds for the requested `PollEvents`, the result is provided on the `PollFd`s
/// if the syscall exits without error.
/// We're doing some translation here, a None `timespec` means forever. A none `SigSetT`
/// means no manipulation. Equivalent usage to regular `poll` syscall using these arguments can
/// be found in the below docs.
/// See the [Linux documentation here](https://man7.org/linux/man-pages/man2/poll.2.html)
/// # Errors
/// See above docs
pub fn ppoll(
    poll_fds: &mut [PollFd],
    timespec: Option<&TimeSpec>,
    sigset: Option<&SigSetT>,
) -> crate::Result<usize> {
    let res = unsafe {
        syscall!(
            PPOLL,
            poll_fds.as_mut_ptr(),
            poll_fds.len(),
            timespec.map_or_else(core::ptr::null, |ts| ts as *const TimeSpec),
            sigset.map_or_else(core::ptr::null, |ss_t| ss_t as *const SigSetT)
        ) as isize
    };
    bail_on_below_zero!(res, "`PPOLL` syscall failed");
    Ok(res as usize)
}

#[cfg(test)]
mod tests {
    use crate::platform::STDOUT;
    use crate::select::{ppoll, PollEvents, PollFd};

    #[test]
    fn dummy() {
        // Stdout should essentially always be ready for output
        let mut poll_fds = [PollFd::new(STDOUT, PollEvents::POLLOUT)];
        let num_rdy = ppoll(&mut poll_fds, None, None).unwrap();
        // Get one changed revents
        assert_eq!(1, num_rdy);
        // Should be pollout
        assert_ne!(
            PollEvents::empty(),
            poll_fds[0].revents & PollEvents::POLLOUT
        );
    }
}
