use sc::syscall;

use crate::string::unix_str::UnixStr;
use crate::Error;

/// Executes provided binary `bin` with arguments `arg_v` and environment `env_p`.
/// Both `arg_v` and `env_p` are null terminated arrays of null terminated strings.
/// They can both be null without unsafety, although common practice is always supplying the binary
/// itself as the first argument.
/// Only returns on error.
/// See [Linux documentation for details](https://man7.org/linux/man-pages/man2/execve.2.html)
/// # Errors
/// See above
/// # Safety
/// See above
#[inline]
pub unsafe fn execve(
    bin: &UnixStr,
    arg_v: *const *const u8,
    env_p: *const *const u8,
) -> Result<(), Error> {
    let res = syscall!(EXECVE, bin.as_ptr(), arg_v, env_p);
    // EXECVE doesn't return on success, on err it returns an error code
    // [docs](https://man7.org/linux/man-pages/man2/execve.2.html#RETURN_VALUE)
    Err(Error::with_code("`EXECVE` syscall failed", res as i32))
}
