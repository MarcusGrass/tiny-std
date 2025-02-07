use core::time::Duration;

use rusl::{
    error::Errno,
    platform::{NonNegativeI32, PollEvents, PollFd, TimeSpec},
};

pub(crate) fn sock_nonblock_op_poll_if_not_ready<
    T,
    F: Fn(NonNegativeI32) -> Result<T, rusl::Error>,
>(
    sock: NonNegativeI32,
    block_errno: Errno,
    ready_event: PollEvents,
    timeout: Option<Duration>,
    op: F,
) -> Result<T, crate::Error> {
    let ts = if let Some(to) = timeout {
        Some(TimeSpec::try_from(to)?)
    } else {
        None
    };
    match op(sock) {
        Ok(o) => Ok(o),
        Err(e) if e.code == Some(block_errno) => {
            let pollfd = PollFd::new(sock, ready_event);
            loop {
                match rusl::select::ppoll(&mut [pollfd], ts.as_ref(), None) {
                    Ok(s) => {
                        if s == 0 {
                            return Err(crate::Error::Timeout);
                        }
                        return op(sock).map_err(crate::Error::from);
                    }
                    Err(e) if e.code == Some(Errno::EINTR) => continue,
                    Err(e) => {
                        return Err(e.into());
                    }
                }
            }
        }
        Err(e) => Err(e.into()),
    }
}

pub(crate) fn blocking_read_nonblock_sock(
    sock: NonNegativeI32,
    buf: &mut [u8],
    timeout: Option<Duration>,
) -> Result<usize, crate::Error> {
    let ts = if let Some(to) = timeout {
        Some(TimeSpec::try_from(to)?)
    } else {
        None
    };
    match rusl::unistd::read(sock, buf) {
        Ok(bytes) => Ok(bytes),
        Err(e) if e.code == Some(Errno::EAGAIN) => {
            let pollfd = PollFd::new(sock, PollEvents::POLLIN);
            loop {
                match rusl::select::ppoll(&mut [pollfd], ts.as_ref(), None) {
                    Ok(s) => {
                        if s == 0 {
                            return Err(crate::Error::Timeout);
                        }
                        return Ok(rusl::unistd::read(sock, buf)?);
                    }
                    Err(e) if e.code == Some(Errno::EINTR) => continue,
                    Err(e) => {
                        return Err(e.into());
                    }
                }
            }
        }
        Err(e) => Err(e.into()),
    }
}

pub(crate) fn blocking_write_nonblock_sock(
    sock: NonNegativeI32,
    buf: &[u8],
    timeout: Option<Duration>,
) -> Result<usize, crate::Error> {
    let ts = if let Some(to) = timeout {
        Some(TimeSpec::try_from(to)?)
    } else {
        None
    };
    match rusl::unistd::write(sock, buf) {
        Ok(bytes) => Ok(bytes),
        Err(e) if e.code == Some(Errno::EAGAIN) => {
            let pollfd = PollFd::new(sock, PollEvents::POLLOUT);
            loop {
                match rusl::select::ppoll(&mut [pollfd], ts.as_ref(), None) {
                    Ok(s) => {
                        if s == 0 {
                            return Err(crate::Error::Timeout);
                        }
                        return Ok(rusl::unistd::write(sock, buf)?);
                    }
                    Err(e) if e.code == Some(Errno::EINTR) => continue,
                    Err(e) => {
                        return Err(e.into());
                    }
                }
            }
        }
        Err(e) => Err(e.into()),
    }
}
