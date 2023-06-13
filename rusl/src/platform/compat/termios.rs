use linux_rust_bindings::termios::tcflag_t;

pub use linux_rust_bindings::termios::{ECHO, ECHONL};

transparent_bitflags! {
    pub struct TermioFlags: usize {
        const DEFAULT = 0;
        const TIOCEXCL = linux_rust_bindings::termios::TIOCEXCL as usize;
        const TIOCNXCL = linux_rust_bindings::termios::TIOCNXCL as usize;
        const TIOCSCTTY = linux_rust_bindings::termios::TIOCSCTTY as usize;
        const TIOCGPGRP = linux_rust_bindings::termios::TIOCGPGRP as usize;
        const TIOCSPGRP = linux_rust_bindings::termios::TIOCSPGRP as usize;
        const TIOCOUTQ = linux_rust_bindings::termios::TIOCOUTQ as usize;
        const TIOCSTI = linux_rust_bindings::termios::TIOCSTI as usize;
        const TIOCGWINSZ = linux_rust_bindings::termios::TIOCGWINSZ as usize;
        const TIOCSWINSZ = linux_rust_bindings::termios::TIOCSWINSZ as usize;
        const TIOCMGET = linux_rust_bindings::termios::TIOCMGET as usize;
        const TIOCMBIS = linux_rust_bindings::termios::TIOCMBIS as usize;
        const TIOCMBIC = linux_rust_bindings::termios::TIOCMBIC as usize;
        const TIOCMSET = linux_rust_bindings::termios::TIOCMSET as usize;
        const TIOCGSOFTCAR = linux_rust_bindings::termios::TIOCGSOFTCAR as usize;
        const TIOCSSOFTCAR = linux_rust_bindings::termios::TIOCSSOFTCAR as usize;
        const TIOCINQ = linux_rust_bindings::termios::TIOCINQ as usize;
        const TIOCLINUX = linux_rust_bindings::termios::TIOCLINUX as usize;
        const TIOCCONS = linux_rust_bindings::termios::TIOCCONS as usize;
        const TIOCGSERIAL = linux_rust_bindings::termios::TIOCGSERIAL as usize;
        const TIOCSSERIAL = linux_rust_bindings::termios::TIOCSSERIAL as usize;
        const TIOCPKT = linux_rust_bindings::termios::TIOCPKT as usize;
        const TIOCNOTTY = linux_rust_bindings::termios::TIOCNOTTY as usize;
        const TIOCSETD = linux_rust_bindings::termios::TIOCSETD as usize;
        const TIOCGETD = linux_rust_bindings::termios::TIOCGETD as usize;
        const TCSBRKP = linux_rust_bindings::termios::TCSBRKP as usize;
        const TIOCSBRK = linux_rust_bindings::termios::TIOCSBRK as usize;
        const TIOCCBRK = linux_rust_bindings::termios::TIOCCBRK as usize;
        const TIOCGSID = linux_rust_bindings::termios::TIOCGSID as usize;
        const TIOCGRS485 = linux_rust_bindings::termios::TIOCGRS485 as usize;
        const TIOCSRS485 = linux_rust_bindings::termios::TIOCSRS485 as usize;
        const TIOCGPTN = tio_shim::TIOCGPTN as usize;
        const TIOCSPTLCK = tio_shim::TIOCSPTLCK as usize;
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
        const DEFAULT = 0;
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
