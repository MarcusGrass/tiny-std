use linux_rust_bindings::termios::tcflag_t;

pub use linux_rust_bindings::termios::{ECHO, ECHONL};

transparent_bitflags! {
    pub struct TermioFlags: u64 {
        const TIOCEXCL = linux_rust_bindings::termios::TIOCEXCL as u64;
        const TIOCNXCL = linux_rust_bindings::termios::TIOCNXCL as u64;
        const TIOCSCTTY = linux_rust_bindings::termios::TIOCSCTTY as u64;
        const TIOCGPGRP = linux_rust_bindings::termios::TIOCGPGRP as u64;
        const TIOCSPGRP = linux_rust_bindings::termios::TIOCSPGRP as u64;
        const TIOCOUTQ = linux_rust_bindings::termios::TIOCOUTQ as u64;
        const TIOCSTI = linux_rust_bindings::termios::TIOCSTI as u64;
        const TIOCGWINSZ = linux_rust_bindings::termios::TIOCGWINSZ as u64;
        const TIOCSWINSZ = linux_rust_bindings::termios::TIOCSWINSZ as u64;
        const TIOCMGET = linux_rust_bindings::termios::TIOCMGET as u64;
        const TIOCMBIS = linux_rust_bindings::termios::TIOCMBIS as u64;
        const TIOCMBIC = linux_rust_bindings::termios::TIOCMBIC as u64;
        const TIOCMSET = linux_rust_bindings::termios::TIOCMSET as u64;
        const TIOCGSOFTCAR = linux_rust_bindings::termios::TIOCGSOFTCAR as u64;
        const TIOCSSOFTCAR = linux_rust_bindings::termios::TIOCSSOFTCAR as u64;
        const TIOCINQ = linux_rust_bindings::termios::TIOCINQ as u64;
        const TIOCLINUX = linux_rust_bindings::termios::TIOCLINUX as u64;
        const TIOCCONS = linux_rust_bindings::termios::TIOCCONS as u64;
        const TIOCGSERIAL = linux_rust_bindings::termios::TIOCGSERIAL as u64;
        const TIOCSSERIAL = linux_rust_bindings::termios::TIOCSSERIAL as u64;
        const TIOCPKT = linux_rust_bindings::termios::TIOCPKT as u64;
        const TIOCNOTTY = linux_rust_bindings::termios::TIOCNOTTY as u64;
        const TIOCSETD = linux_rust_bindings::termios::TIOCSETD as u64;
        const TIOCGETD = linux_rust_bindings::termios::TIOCGETD as u64;
        const TCSBRKP = linux_rust_bindings::termios::TCSBRKP as u64;
        const TIOCSBRK = linux_rust_bindings::termios::TIOCSBRK as u64;
        const TIOCCBRK = linux_rust_bindings::termios::TIOCCBRK as u64;
        const TIOCGSID = linux_rust_bindings::termios::TIOCGSID as u64;
        const TIOCGRS485 = linux_rust_bindings::termios::TIOCGRS485 as u64;
        const TIOCSRS485 = linux_rust_bindings::termios::TIOCSRS485 as u64;
        const TIOCGPTN = tio_shim::TIOCGPTN as u64;
        const TIOCSPTLCK = tio_shim::TIOCSPTLCK as u64;
    }
}
#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct WindowSize(linux_rust_bindings::termios::winsize);

impl WindowSize {
    #[must_use]
    pub const fn new(row: u16, col: u16, x_pixel: u16, y_pixel: u16) -> Self {
        Self(linux_rust_bindings::termios::winsize {
            ws_row: row,
            ws_col: col,
            ws_xpixel: x_pixel,
            ws_ypixel: y_pixel,
        })
    }
}

impl Default for WindowSize {
    #[inline]
    fn default() -> Self {
        Self::new(0, 0, 0, 0)
    }
}

transparent_bitflags! {
    pub struct InputFlags: tcflag_t {
        const IGNBRK = linux_rust_bindings::termios::IGNBRK as tcflag_t;
        const BRKINT = linux_rust_bindings::termios::BRKINT as tcflag_t;
        const IGNPAR = linux_rust_bindings::termios::IGNPAR as tcflag_t;
        const PARMRK = linux_rust_bindings::termios::PARMRK as tcflag_t;
        const INPCK = linux_rust_bindings::termios::INPCK as tcflag_t;
        const ISTRIP = linux_rust_bindings::termios::ISTRIP as tcflag_t;
        const INLCR = linux_rust_bindings::termios::INLCR as tcflag_t;
        const IGNCR = linux_rust_bindings::termios::IGNCR as tcflag_t;
        const ICRNL = linux_rust_bindings::termios::ICRNL as tcflag_t;
        const IXON = linux_rust_bindings::termios::IXON as tcflag_t;
        const IXOFF = linux_rust_bindings::termios::IXOFF as tcflag_t;
        const IXANY = linux_rust_bindings::termios::IXANY as tcflag_t;
        const IMAXBEL = linux_rust_bindings::termios::IMAXBEL as tcflag_t;
        const IUTF8 = linux_rust_bindings::termios::IUTF8 as tcflag_t;
    }
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct Termios(pub linux_rust_bindings::termios::termios2);

impl Termios {
    pub fn set_iflag(&mut self, flags: InputFlags, state: bool) {
        if state {
            self.0.c_iflag |= flags.bits();
        } else {
            self.0.c_iflag &= !flags.bits();
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum SetAction {
    NOW,
    DRAIN,
    FLUSH,
}

impl SetAction {
    pub(crate) const fn into_i32(self) -> i32 {
        match self {
            SetAction::NOW => linux_rust_bindings::termios::TCSANOW,
            SetAction::DRAIN => linux_rust_bindings::termios::TCSADRAIN,
            SetAction::FLUSH => linux_rust_bindings::termios::TCSAFLUSH,
        }
    }
}

pub(crate) mod tio_shim {
    pub const TIOCGPTN: u32 = crate::_ior!('T' as u32, 0x30u32, u32);
    pub const TIOCSPTLCK: i32 = crate::_iow!('T' as i32, 0x31i32, i32);

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_macro_compat() {
            assert_eq!(TIOCGPTN, 0x80045430);
            assert_eq!(TIOCSPTLCK, 0x40045431);
        }
    }
}
