use crate::Error;
use sc::syscall;

use crate::platform::{Fd, OpenFlags};

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
    pipe2(OpenFlags::empty())
}

/// Creates a new set of pipes by utilizing the `PIPE2` syscall, with openflags
/// See that [documentation here](https://man7.org/linux/man-pages/man2/pipe.2.html)
/// # Errors
/// See above
#[inline]
pub fn pipe2(flags: OpenFlags) -> crate::Result<Pipe> {
    let mut fds = [-1, -1];
    let res = unsafe { syscall!(PIPE2, fds.as_mut_ptr(), flags.bits().0) };
    bail_on_below_zero!(res, "`PIPE2` syscall failed creating new pipe");
    Ok(Pipe {
        in_pipe: Fd::try_new(fds[0]).map_err(|_e| Error::no_code("In pipe fd below zero"))?,
        out_pipe: Fd::try_new(fds[1]).map_err(|_e| Error::no_code("Out pipe fd below zero"))?,
    })
}

#[cfg(test)]
mod tests {
    use crate::unistd::pipe;

    #[test]
    fn test_pipe() {
        const PAYLOAD: &[u8; 6] = b"Hello!";
        let pipe = pipe().unwrap();

        crate::unistd::write(pipe.out_pipe, PAYLOAD).unwrap();
        let mut buf = [0u8; 6];
        crate::unistd::read(pipe.in_pipe, &mut buf).unwrap();
        assert_eq!(PAYLOAD, &buf);
    }
}
