use sc::syscall;

use crate::platform::{EpollEvent, EpollOp, Fd};
use crate::Result;

/// Create an epoll fd
/// See [linux documentation for details](https://man7.org/linux/man-pages/man2/epoll_create.2.html)
/// # Errors
/// See above
#[inline]
pub fn epoll_create(cloexec: bool) -> Result<Fd> {
    let flags = if cloexec {
        linux_rust_bindings::epoll::EPOLL_CLOEXEC
    } else {
        0
    };
    let res = unsafe { syscall!(EPOLL_CREATE1, flags) };
    bail_on_below_zero!(res, "`EPOLL_CREATE1` syscall failed");
    Ok(res as Fd)
}

/// Add, remove, or modify the interest list of the specified `epoll_fd`.
/// See [Linux documentation for details(https://man7.org/linux/man-pages/man2/epoll_ctl.2.html)
/// # Errors
/// See above
#[inline]
pub fn epoll_ctl(epoll_fd: Fd, epoll_op: EpollOp, fd: Fd, event: &EpollEvent) -> Result<()> {
    let evt_addr = core::ptr::addr_of!(event.0);
    let res = unsafe { syscall!(EPOLL_CTL, epoll_fd, epoll_op.into_op(), fd, evt_addr) };
    bail_on_below_zero!(res, "`EPOLL_CTL` syscall failed");
    Ok(())
}

/// Remove an fd from the epoll interest list.
/// Some duplication with above, the `event` argument isn't needed for delete since
/// kernel 2.6.9, if targeting a kernel earlier than that, use the above function
/// See [Linux documentation for details(https://man7.org/linux/man-pages/man2/epoll_ctl.2.html)
/// # Errors
/// See above
#[inline]
pub fn epoll_del(epoll_fd: Fd, fd: Fd) -> Result<()> {
    let res = unsafe { syscall!(EPOLL_CTL, epoll_fd, EpollOp::Del.into_op(), fd, 0) };
    bail_on_below_zero!(res, "`EPOLL_CTL` syscall failed");
    Ok(())
}

/// Wait for an `epoll_fd` to have ready events for as most `timeout_millis`.
/// `timeout_millis = -1` means wait until an event fires.
/// `timeout_millis = 0` means return immediately, even if no events are ready.
/// See the [Linux documentation for details](https://man7.org/linux/man-pages/man2/epoll_wait.2.html)
/// # Errors
/// See above
#[inline]
pub fn epoll_wait(epoll_fd: Fd, events: &mut [EpollEvent], timeout_millis: i32) -> Result<usize> {
    let res = unsafe {
        syscall!(
            EPOLL_PWAIT,
            epoll_fd,
            events.as_mut_ptr(),
            events.len(),
            timeout_millis,
            0,
            0
        )
    };
    bail_on_below_zero!(res, "`EPOLL_PWAIT` syscall failed");
    Ok(res)
}

#[cfg(test)]
mod test {
    use crate::platform::{EpollEvent, EpollEventMask, EpollOp, STDIN};
    use crate::select::{epoll_create, epoll_ctl, epoll_wait};

    #[test]
    fn can_setup_wait() {
        let epoll_fd = epoll_create(true).unwrap();
        epoll_ctl(
            epoll_fd,
            EpollOp::Add,
            STDIN,
            &EpollEvent::new(15, EpollEventMask::EPOLLIN | EpollEventMask::EPOLLOUT),
        )
        .unwrap();
        let mut ret = [EpollEvent::new(0, EpollEventMask::empty())];
        let ready = epoll_wait(epoll_fd, &mut ret, 10).unwrap();
        assert_eq!(1, ready);
        assert_eq!(15, ret[0].get_data());
        assert_eq!(EpollEventMask::EPOLLOUT, ret[0].get_events());
    }
}
