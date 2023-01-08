use linux_rust_bindings::io_uring::{IORING_REGISTER_BUFFERS, IORING_REGISTER_FILES};
use sc::syscall;

use crate::platform::{Fd, IoSliceMut, IoUringParams};
use crate::Result;

/// Sets up a new `io_uring` instance fitting `entries` amount of entries
/// returning its `fd`.
/// See [Linux documentation for details]()
/// # Errors
/// See above
#[inline]
pub fn io_uring_setup(entries: u32, io_uring_params: &mut IoUringParams) -> Result<Fd> {
    let res = unsafe {
        syscall!(
            IO_URING_SETUP,
            entries,
            io_uring_params as *mut IoUringParams
        )
    };
    bail_on_below_zero!(res, "`IO_URING_SETUP` syscall failed");
    Ok(res as Fd)
}

/// Register files on an `io_uring` instance.
/// See [Linux documentation for details]()
/// # Errors
/// See above
#[inline]
pub fn io_uring_register_files(uring_fd: Fd, fds: &[Fd]) -> Result<()> {
    let res = unsafe {
        syscall!(
            IO_URING_REGISTER,
            uring_fd,
            IORING_REGISTER_FILES,
            fds.as_ptr(),
            fds.len()
        )
    };
    bail_on_below_zero!(res, "`IO_URING_REGISTER` Syscall failed registering files");
    Ok(())
}

/// Register io slices on an `io_uring` instance.
/// See [Linux documentation for details]()
/// # Errors
/// See above
#[inline]
pub fn io_uring_register_io_slices(uring_fd: Fd, buffers: &[IoSliceMut]) -> Result<()> {
    let res = unsafe {
        syscall!(
            IO_URING_REGISTER,
            uring_fd,
            IORING_REGISTER_BUFFERS,
            buffers.as_ptr(),
            buffers.len()
        )
    };
    bail_on_below_zero!(
        res,
        "`IO_URING_REGISTER` Syscall failed registering io slices"
    );
    Ok(())
}

/// Register a fixed buffer on an `io_uring` instance.
/// See [Linux documentation for details]()
/// # Errors
/// See above
#[inline]
pub fn io_uring_register_buf(uring_fd: Fd, buffer: &[u8]) -> Result<()> {
    let res = unsafe {
        syscall!(
            IO_URING_REGISTER,
            uring_fd,
            IORING_REGISTER_BUFFERS,
            buffer.as_ptr(),
            1
        )
    };
    bail_on_below_zero!(res, "`IO_URING_REGISTER` Syscall failed registering buffer");
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::error::Errno;
    use crate::io_uring::{
        io_uring_register_buf, io_uring_register_files, io_uring_register_io_slices, io_uring_setup,
    };
    use crate::platform::{Fd, IoSliceMut, IoUringParamFlags, IoUringParams, OpenFlags};
    use crate::unistd::open;

    #[test]
    fn uring_setup() {
        let _ = setup_io_poll_uring();
    }

    fn setup_io_poll_uring() -> Option<Fd> {
        let mut params = IoUringParams::new(IoUringParamFlags::IORING_SETUP_IOPOLL);
        let uring_fd = match io_uring_setup(1, &mut params) {
            Ok(uring_fd) => {
                assert_ne!(0, uring_fd);
                uring_fd
            }
            Err(e) => {
                if e.code.unwrap() != Errno::ENOSYS {
                    panic!("{}", e);
                }
                return None;
            }
        };
        Some(uring_fd)
    }

    #[test]
    fn uring_register_files() {
        let Some(uring_fd) = setup_io_poll_uring() else {
            return;
        };
        let open_fd = open("test-files/can_open.txt\0", OpenFlags::O_RDWR).unwrap();
        io_uring_register_files(uring_fd, &[open_fd]).unwrap();
    }

    #[test]
    fn uring_register_io_slices() {
        let Some(uring_fd) = setup_io_poll_uring() else {
            return;
        };
        let mut buf1 = [0; 1024];
        let mut buf2 = [0; 1024];
        let io_slices = [IoSliceMut::new(&mut buf1), IoSliceMut::new(&mut buf2)];
        io_uring_register_io_slices(uring_fd, &io_slices).unwrap();
    }

    #[test]
    fn uring_register_buffer() {
        let Some(uring_fd) = setup_io_poll_uring() else {
            return;
        };
        let buf1 = [0; 1024];
        io_uring_register_buf(uring_fd, &buf1).unwrap();
    }
}
