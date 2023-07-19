pub use crate::platform::numbers::NonNegativeI32;
pub use auxvec::*;
pub use clone::*;
pub use dirent::*;
pub use elf::*;
pub use epoll::*;
pub use fcntl::*;
pub use futex::*;
pub use hidio::*;
pub use io_uring::*;
pub use mman::*;
pub use mount::*;
pub use poll::*;
pub use renameat::*;
pub use signal::*;
pub use socket::*;
pub use stat::*;
pub use termios::*;
pub use time::*;
pub use uio::*;
pub use usb::*;
pub use utsname::*;
pub use wait::*;

mod auxvec;
mod clone;
mod dirent;
mod elf;
mod epoll;
mod fcntl;
mod futex;
mod hidio;
mod io_uring;
mod mman;
mod mount;
mod poll;
mod renameat;
mod signal;
mod socket;
mod stat;
mod termios;
mod time;
mod uio;
mod usb;
mod utsname;
mod wait;

/// Shared typedefs for 64 bit systems (GNU source)
pub type UidT = u32;
pub type GidT = u32;
pub type PidT = i32;
pub type TidT = i32;
pub type Fd = NonNegativeI32;
pub type OffT = i64;
pub type BlkSizeT = i64;
pub type BlkCntT = i64;
pub type DevT = u64;
pub type InoT = u64;
pub type Ino64T = u64;
pub type NlinkT = u64;

pub const NULL_BYTE: u8 = b'\0';
pub const NULL_CHAR: char = '\0';

pub const STDIN: Fd = NonNegativeI32::comptime_checked_new(0);
pub const STDOUT: Fd = NonNegativeI32::comptime_checked_new(1);
pub const STDERR: Fd = NonNegativeI32::comptime_checked_new(2);

const LINUX_ERROR_RESV: usize = 4095;
const SYSCALL_ERR_THRESHOLD: usize = usize::MAX - LINUX_ERROR_RESV;

#[must_use]
#[inline(always)]
#[allow(clippy::inline_always)]
pub const fn is_syscall_error(res: usize) -> bool {
    res > SYSCALL_ERR_THRESHOLD
}

/// For this to be syscall compatible, the generated i32 needs to be downgraded to u8
transparent_bitflags! {
    pub struct DirType: u8 {
        const DEFAULT = 0;
        const DT_UNKNOWN = comptime_i32_to_u8(linux_rust_bindings::types::DT_UNKNOWN);
        const DT_FIFO = comptime_i32_to_u8(linux_rust_bindings::types::DT_FIFO);
        const DT_CHR = comptime_i32_to_u8(linux_rust_bindings::types::DT_CHR);
        const DT_DIR = comptime_i32_to_u8(linux_rust_bindings::types::DT_DIR);
        const DT_BLK = comptime_i32_to_u8(linux_rust_bindings::types::DT_BLK);
        const DT_REG = comptime_i32_to_u8(linux_rust_bindings::types::DT_REG);
        const DT_LNK = comptime_i32_to_u8(linux_rust_bindings::types::DT_LNK);
        const DT_SOCK = comptime_i32_to_u8(linux_rust_bindings::types::DT_SOCK);
    }
}

#[macro_export]
macro_rules! _ioc {
    ($dir:expr, $io_ty: expr, $nr: expr, $sz: expr, $ty: ty) => {
        ($dir << linux_rust_bindings::ioctl::_IOC_DIRSHIFT as $ty)
            | ($io_ty << linux_rust_bindings::ioctl::_IOC_TYPESHIFT as $ty)
            | ($nr << linux_rust_bindings::ioctl::_IOC_NRSHIFT as $ty)
            | ($sz << linux_rust_bindings::ioctl::_IOC_SIZESHIFT as $ty)
    };
}

#[macro_export]
macro_rules! _ior {
    ($io_ty: expr, $nr: expr, $ty: ty) => {
        $crate::_ioc!(
            linux_rust_bindings::ioctl::_IOC_READ as $ty,
            $io_ty,
            $nr,
            core::mem::size_of::<$ty>() as $ty,
            $ty
        )
    };
}

#[macro_export]
macro_rules! _iow {
    ($io_ty: expr, $nr: expr, $ty: ty) => {
        $crate::_ioc!(
            linux_rust_bindings::ioctl::_IOC_WRITE as $ty,
            $io_ty,
            $nr,
            core::mem::size_of::<$ty>() as $ty,
            $ty
        )
    };
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub(crate) const fn comptime_i32_to_u8(input: i32) -> u8 {
    assert!(
        input >= 0 && input < u8::MAX as i32,
        "Provided i32 cannot be converted to a u8"
    );
    input as u8
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub(crate) const fn comptime_i32_to_u32(input: i32) -> u32 {
    assert!(input >= 0, "Provided i32 cannot be converted to a u32");
    input as u32
}

#[allow(clippy::cast_possible_truncation)]
pub(crate) const fn comptime_i32_to_i16(input: i32) -> i16 {
    assert!(
        input < i16::MAX as i32 && input > i16::MIN as i32,
        "Provided i32 cannot be converted to an i16"
    );
    input as i16
}

#[allow(clippy::cast_possible_truncation)]
pub(crate) const fn comptime_u32_to_u8(input: u32) -> u8 {
    assert!(
        input < u8::MAX as u32,
        "Provided u32 cannot be converted to a u8"
    );
    input as u8
}

#[cfg(test)]
mod tests {
    #[test]
    fn dummy() {}
}
