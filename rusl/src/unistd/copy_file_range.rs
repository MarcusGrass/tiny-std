use crate::platform::Fd;
use sc::syscall;

/// Copies from one file-offset to another in-kernel
/// # Errors
/// See the [linux docs for details](https://man7.org/linux/man-pages/man2/copy_file_range.2.html#ERRORS)
pub fn copy_file_range(
    src_fd: Fd,
    src_offset: u64,
    dest_fd: Fd,
    dest_offset: u64,
    len: usize,
) -> crate::Result<usize> {
    let res = unsafe {
        syscall!(
            COPY_FILE_RANGE,
            src_fd.value(),
            src_offset,
            dest_fd.value(),
            dest_offset,
            len,
            0
        )
    };
    bail_on_below_zero!(res, "`COPY_FILE_RANGE` syscall failed");
    Ok(res)
}
