use core::mem::MaybeUninit;

use sc::syscall;

use crate::platform::UtsName;

/// Gets the `UtsName` struct
/// See the [linux docs for details](https://man7.org/linux/man-pages/man2/uname.2.html)
/// # Errors
/// See above
pub fn uname() -> crate::Result<UtsName> {
    let mut uts_name = MaybeUninit::uninit();
    let res = unsafe { syscall!(UNAME, uts_name.as_mut_ptr()) };
    bail_on_below_zero!(res, "Failed `UNAME` syscall");
    unsafe { Ok(uts_name.assume_init()) }
}

#[cfg(test)]
mod tests {
    use unix_print::unix_eprintln;

    use crate::unistd::uname::uname;

    #[test]
    fn test_uts_name() {
        let uts_name = uname().unwrap();
        let host_name = uts_name.nodename().unwrap();
        unix_eprintln!("{host_name:?}");
    }
}
