#[cfg(target_arch = "aarch64")]
pub use aarch64::*;
pub use error_codes::*;
pub use shared_64::*;
#[cfg(target_arch = "x86_64")]
pub use x86_64::*;

#[cfg(target_arch = "aarch64")]
mod aarch64;
pub mod error_codes;
#[cfg(target_arch = "x86_64")]
mod x86_64;

mod shared_64;
pub mod vdso;

pub const NULL_BYTE: u8 = b'\0';
pub const NULL_CHAR: char = '\0';

pub const STDIN: Fd = 0;
pub const STDOUT: Fd = 1;
pub const STDERR: Fd = 2;

#[cfg(test)]
mod tests {
    #[test]
    fn dummy() {}
}
