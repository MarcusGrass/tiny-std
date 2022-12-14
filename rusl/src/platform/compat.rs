pub use auxvec::*;
pub use clone::*;
pub use dirent::*;
pub use epoll::*;
pub use fcntl::*;
pub use io_uring::*;
pub use mman::*;
pub use mode_flags::*;
pub use open_flags::*;
pub use poll::*;
pub use signal::*;
pub use socket::*;
pub use stat::*;
pub use termios::*;
pub use time::*;
pub use uio::*;
pub use utsname::*;
pub use vdso::*;
pub use wait::*;

mod auxvec;
mod clone;
mod dirent;
mod epoll;
mod fcntl;
mod io_uring;
mod mman;
mod mode_flags;
mod open_flags;
mod poll;
mod signal;
mod socket;
mod stat;
mod termios;
mod time;
mod uio;
mod utsname;
mod vdso;
mod wait;

/// Shared typedefs for 64 bit systems (GNU source)
pub type UidT = u32;
pub type GidT = u32;
pub type PidT = i32;
pub type TidT = i32;
pub type Fd = i32;
pub type OffT = i64;
pub type BlkSizeT = i64;
pub type BlkCntT = i64;
pub type DevT = u64;
pub type InoT = u64;
pub type Ino64T = u64;
pub type NlinkT = u64;

pub const NULL_BYTE: u8 = b'\0';
pub const NULL_CHAR: char = '\0';

pub const STDIN: Fd = 0;
pub const STDOUT: Fd = 1;
pub const STDERR: Fd = 2;

/// For this to be syscall compatible, the generated i32 needs to be downgraded to u8
transparent_bitflags! {
    pub struct DirType: u8 {
        const DT_UNKNOWN = linux_rust_bindings::types::DT_UNKNOWN as u8;
        const DT_FIFO = linux_rust_bindings::types::DT_FIFO as u8;
        const DT_CHR = linux_rust_bindings::types::DT_CHR as u8;
        const DT_DIR = linux_rust_bindings::types::DT_DIR as u8;
        const DT_BLK = linux_rust_bindings::types::DT_BLK as u8;
        const DT_REG = linux_rust_bindings::types::DT_REG as u8;
        const DT_LNK = linux_rust_bindings::types::DT_LNK as u8;
        const DT_SOCK = linux_rust_bindings::types::DT_SOCK as u8;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn dummy() {}
}
