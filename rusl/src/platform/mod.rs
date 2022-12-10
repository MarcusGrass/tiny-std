pub use shared_64::*;

mod shared_64;
pub use linux_rust_bindings::*;
pub type Stat = stat;

pub const AUX_CNT: usize = 38;
pub const NULL_BYTE: u8 = b'\0';
pub const NULL_CHAR: char = '\0';

pub const STDIN: Fd = 0;
pub const STDOUT: Fd = 1;
pub const STDERR: Fd = 2;

#[cfg(test)]
mod tests {
    #[test]
    fn dummy() {
    }
}
