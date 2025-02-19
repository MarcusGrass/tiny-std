use core::num::NonZeroUsize;

use sc::syscall;

use crate::platform::{Fd, MapAdditionalFlags, MapRequiredFlag, MemoryProtection, OffT};

/// Map files or devices into memory.
/// Almost impossible to make safe, and the [linux documentation](https://man7.org/linux/man-pages/man2/mmap.2.html)
/// should be consulted for details.
/// # Errors
/// See above
/// # Safety
/// see above
pub unsafe fn mmap(
    addr: Option<usize>,
    length: NonZeroUsize,
    memory_protection: MemoryProtection,
    required_flag: MapRequiredFlag,
    additional_flags: MapAdditionalFlags,
    fd: Option<Fd>,
    offset: OffT,
) -> crate::Result<usize> {
    let flags = required_flag.into_flag() | additional_flags;
    let res_ptr = syscall!(
        MMAP,
        addr.unwrap_or_default(),
        length.get(),
        memory_protection.bits(),
        flags.bits(),
        fd.map_or(-1, |fd| fd.0),
        offset
    );
    bail_on_below_zero!(res_ptr, "`MMAP` syscall failed");
    Ok(res_ptr)
}

/// Unmaps memory.
/// Almost impossible to make safe, and the [linux documentation](https://man7.org/linux/man-pages/man2/mmap.2.html)
/// should be consulted for details.
/// # Errors
/// See above
/// # Safety
/// see above
#[inline]
pub unsafe fn munmap(addr: usize, length: NonZeroUsize) -> crate::Result<()> {
    let res_ptr = syscall!(MUNMAP, addr, length.get());
    bail_on_below_zero!(res_ptr, "`MUNMAP` syscall failed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // We're using the truncation in the test
    #[expect(clippy::cast_possible_truncation)]
    fn test_basic_alloc() {
        unsafe {
            let size = 4096;
            let sz = NonZeroUsize::new(size).unwrap();
            let stack = mmap(
                None,
                sz,
                MemoryProtection::PROT_READ | MemoryProtection::PROT_WRITE,
                MapRequiredFlag::MapPrivate,
                MapAdditionalFlags::MAP_ANONYMOUS,
                None,
                0,
            )
            .unwrap();
            let slice_stack: &mut [u8] = core::slice::from_raw_parts_mut(stack as _, size);
            for (i, val) in slice_stack.iter_mut().enumerate() {
                // The memory should be zeroed
                assert_eq!(0, *val);
                // The memory should be writeable
                *val = i as u8;
            }
            for (i, val) in slice_stack.iter().enumerate() {
                assert_eq!(i as u8, *val);
            }
            munmap(stack, sz).unwrap();
        }
    }
}
