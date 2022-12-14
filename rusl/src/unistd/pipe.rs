use sc::syscall;

use crate::platform::Fd;
use crate::unistd::OpenFlags;

#[derive(Debug, Copy, Clone)]
pub struct Pipe {
    pub in_pipe: Fd,
    pub out_pipe: Fd,
}

/// Creates a new set of pipes by utilizing the `PIPE2` syscall
/// See that [documentation here](https://man7.org/linux/man-pages/man2/pipe.2.html)
/// # Errors
/// See above
pub fn pipe() -> crate::Result<Pipe> {
    let mut fds = [-1, -1];
    let res = unsafe { syscall!(PIPE2, fds.as_mut_ptr(), 0) };
    bail_on_below_zero!(res, "`PIPE2` syscall failed creating new pipe");
    Ok(Pipe {
        in_pipe: fds[0],
        out_pipe: fds[1],
    })
}

/// Creates a new set of pipes by utilizing the `PIPE2` syscall, with openflags
/// See that [documentation here](https://man7.org/linux/man-pages/man2/pipe.2.html)
/// # Errors
/// See above
pub fn pipe2(flags: OpenFlags) -> crate::Result<Pipe> {
    let mut fds = [-1, -1];
    let res = unsafe { syscall!(PIPE2, fds.as_mut_ptr(), flags.bits()) };
    bail_on_below_zero!(res, "`PIPE2` syscall failed creating new pipe");
    Ok(Pipe {
        in_pipe: fds[0],
        out_pipe: fds[1],
    })
}
