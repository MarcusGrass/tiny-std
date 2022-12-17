use core::mem::MaybeUninit;

use linux_rust_bindings::termios::TCGETS;
use sc::syscall;

use crate::platform::{Fd, Termios};

/// Gets the attributes of the terminal connected to the supplied `Fd`
/// # Errors
/// See the [Opengroup spec](https://pubs.opengroup.org/onlinepubs/007904975/functions/tcgetattr.html)
pub fn tcgetattr(fd: Fd) -> crate::Result<Termios> {
    let mut termios = MaybeUninit::uninit();
    unsafe {
        let res = syscall!(IOCTL, fd, TCGETS, termios.as_mut_ptr());
        bail_on_below_zero!(res, "`IOCTL` syscall through tcgetattr failed");
        Ok(termios.assume_init())
    }
}
