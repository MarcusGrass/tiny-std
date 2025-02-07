use rusl::platform::{GidT, UidT};
use rusl::string::unix_str::UnixStr;
use tiny_start::elf::aux::AuxValues;

/// A vector of dynamic values supplied by the OS
pub(crate) static mut AUX_VALUES: AuxValues = AuxValues::zeroed();

/// Get the main thread group id
#[inline]
#[must_use]
#[expect(clippy::cast_possible_truncation)]
pub fn get_gid() -> GidT {
    unsafe { AUX_VALUES.at_gid as GidT }
}

/// Get the main thread user id
#[inline]
#[must_use]
#[expect(clippy::cast_possible_truncation)]
pub fn get_uid() -> UidT {
    unsafe { AUX_VALUES.at_uid as UidT }
}

/// Get 16 random bytes of data, run-specific, i.e. does not change between calls.
/// Could call this something like `session-id` to make it more clear that repeated calls
/// won't yield different values.
#[inline]
#[must_use]
pub fn get_random() -> Option<u128> {
    unsafe {
        let random_addr = AUX_VALUES.at_random;
        if random_addr != 0 {
            // This pointer isn't necessarily aligned properly for a u128
            let random_bytes = random_addr as *const u8;
            let unbounded_slice = core::slice::from_raw_parts(random_bytes, 16);
            return Some(u128::from_ne_bytes(
                unbounded_slice.try_into().unwrap_unchecked(),
            ));
        }
        None
    }
}

/// Get the pathname used to execute the program
#[inline]
#[must_use]
pub fn get_exec_fn() -> Option<&'static UnixStr> {
    unsafe {
        let fn_addr = AUX_VALUES.at_execfn;
        if fn_addr != 0 {
            return Some(UnixStr::from_ptr(fn_addr as *const u8));
        }
        None
    }
}
