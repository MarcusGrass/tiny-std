use sc::syscall;

/// Ioctl is a generic IO control interface, it can be used in a myriad of ways
/// and is hard to gurantee safety in.
/// See the [linux docs for details](https://man7.org/linux/man-pages/man2/ioctl.2.html).
/// The return value differs depending on usage.
/// # Errors
/// See above
/// # Safety
/// See above
pub unsafe fn ioctl(a: usize, b: usize, c: usize) -> crate::Result<usize> {
    let res = syscall!(IOCTL, a, b, c);
    bail_on_below_zero!(res, "`IOCTL` syscall failed");
    Ok(res)
}
