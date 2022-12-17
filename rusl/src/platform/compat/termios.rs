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

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct Termios(linux_rust_bindings::termios::termios2);

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

mod tio_shim {
    use linux_rust_bindings::ioctl::{
        _IOC_DIRSHIFT, _IOC_NRSHIFT, _IOC_READ, _IOC_SIZESHIFT, _IOC_TYPESHIFT, _IOC_WRITE,
    };

    macro_rules! _ioc {
        ($dir:expr, $io_ty: expr, $nr: expr, $sz: expr, $ty: ty) => {
            ($dir << _IOC_DIRSHIFT as $ty)
                | ($io_ty << _IOC_TYPESHIFT as $ty)
                | ($nr << _IOC_NRSHIFT as $ty)
                | ($sz << _IOC_SIZESHIFT as $ty)
        };
    }
    macro_rules! _ior {
        ($io_ty: expr, $nr: expr, $ty: ty) => {
            _ioc!(
                _IOC_READ as $ty,
                $io_ty,
                $nr,
                core::mem::size_of::<$ty>() as $ty,
                $ty
            )
        };
    }
    macro_rules! _iow {
        ($io_ty: expr, $nr: expr, $ty: ty) => {
            _ioc!(
                _IOC_WRITE as $ty,
                $io_ty,
                $nr,
                core::mem::size_of::<$ty>() as $ty,
                $ty
            )
        };
    }
    pub const TIOCGPTN: u32 = _ior!('T' as u32, 0x30u32, u32);
    pub const TIOCSPTLCK: i32 = _iow!('T' as i32, 0x31i32, i32);

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
