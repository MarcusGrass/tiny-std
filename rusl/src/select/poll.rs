use sc::syscall;

use crate::platform::{PollFd, SigSetT, TimeSpec};

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
        )
    };
    bail_on_below_zero!(res, "`PPOLL` syscall failed");
    Ok(res)
}

#[cfg(test)]
mod tests {
    use crate::platform::{PollEvents, PollFd, TimeSpec, STDOUT};
    use crate::select::ppoll;

    #[test]
    fn poll_stdout_ready() {
        // Stdout should essentially always be ready for output
        let mut poll_fds = [PollFd::new(STDOUT, PollEvents::POLLOUT)];
        println!("Dummy out");
        let num_rdy = ppoll(&mut poll_fds, Some(&TimeSpec::new(1, 0)), None).unwrap();
        // Get one changed revents
        assert_eq!(1, num_rdy);
        // Should be pollout
        assert_ne!(
            PollEvents::empty(),
            poll_fds[0].received_events() & PollEvents::POLLOUT
        );
    }
}
