use linux_rust_bindings::termios::TCSETS;
use sc::syscall;

use crate::platform::{Fd, SetAction, Termios};

/// Attempts to set attributes to a terminal.
/// # Errors
/// See the [Open group spec](https://pubs.opengroup.org/onlinepubs/009696799/functions/tcsetattr.html)
/// If this isn't a valid terminal `fd`, or an `EINTR`, since we're restricting what values can go in.
pub fn tcsetattr(fd: Fd, action: SetAction, termios: &Termios) -> crate::Result<()> {
    let res = unsafe {
        syscall!(
            IOCTL,
            fd.0,
            TCSETS + action.into_i32(),
            core::ptr::from_ref::<Termios>(termios)
        )
    };
    bail_on_below_zero!(res, "`IOCTL` `TCSETS` syscall failed");
    Ok(())
}
