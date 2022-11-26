use core::hint::unreachable_unchecked;

use sc::syscall;

/// Causes normal process termination and returns the least significant byte
/// of the code to the parent
#[inline]
pub fn exit(code: i32) -> ! {
    unsafe {
        syscall!(EXIT, code);
        unreachable_unchecked();
    }
}
