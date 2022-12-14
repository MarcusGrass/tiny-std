pub use auxvec::*;
pub use socket::*;
pub use time::*;
pub use vdso::*;
mod auxvec;
mod socket;
mod time;
mod vdso;

use crate::platform::Fd;
use linux_rust_bindings::stat;

pub type Stat = stat;

// Got this from `musl` gotta see if I can find it in the kernel source
pub const AUX_CNT: usize = 38;
pub const NULL_BYTE: u8 = b'\0';
pub const NULL_CHAR: char = '\0';

pub const STDIN: Fd = 0;
pub const STDOUT: Fd = 1;
pub const STDERR: Fd = 2;

/// For this to be syscall compatible, the generated i32 needs to be downgraded to u8
transparent_bitflags! {
    pub struct DirType: u8 {
        const DT_UNKNOWN = linux_rust_bindings::DT_UNKNOWN as u8;
        const DT_FIFO = linux_rust_bindings::DT_FIFO as u8;
        const DT_CHR = linux_rust_bindings::DT_CHR as u8;
        const DT_DIR = linux_rust_bindings::DT_DIR as u8;
        const DT_BLK = linux_rust_bindings::DT_BLK as u8;
        const DT_REG = linux_rust_bindings::DT_REG as u8;
        const DT_LNK = linux_rust_bindings::DT_LNK as u8;
        const DT_SOCK = linux_rust_bindings::DT_SOCK as u8;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn dummy() {}
}
