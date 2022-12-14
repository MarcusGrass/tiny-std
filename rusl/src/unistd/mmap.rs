use core::num::NonZeroUsize;
use sc::syscall;
use crate::platform::{Fd, OffT};

transparent_bitflags! {
    pub struct MemoryProtection: i32 {
        const PROT_NONE = linux_rust_bindings::PROT_NONE;
        const PROT_READ = linux_rust_bindings::PROT_READ;
        const PROT_WRITE = linux_rust_bindings::PROT_WRITE;
        const PROT_EXEC = linux_rust_bindings::PROT_EXEC;
        const PROT_SEM = linux_rust_bindings::PROT_SEM;
        const PROT_GROWSDOWN = linux_rust_bindings::PROT_GROWSDOWN;
        const PROT_GROWSUP = linux_rust_bindings::PROT_GROWSUP;
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MapRequiredFlag {
    MapShared,
    MapSharedValidate,
    MapPrivate
}

impl MapRequiredFlag {
    const fn into_flag(self) -> MapAdditionalFlags {
        MapAdditionalFlags(match self {
            MapRequiredFlag::MapShared => linux_rust_bindings::MAP_SHARED,
            MapRequiredFlag::MapSharedValidate => linux_rust_bindings::MAP_SHARED_VALIDATE,
            MapRequiredFlag::MapPrivate => linux_rust_bindings::MAP_PRIVATE,
        })
    }
}

transparent_bitflags! {
    pub struct MapAdditionalFlags: i32 {
        const MAP_TYPE = linux_rust_bindings::MAP_TYPE;
        const MAP_FIXED = linux_rust_bindings::MAP_FIXED;
        const MAP_ANONYMOUS = linux_rust_bindings::MAP_ANONYMOUS;
        const MAP_POPULATE = linux_rust_bindings::MAP_POPULATE;
        const MAP_NONBLOCK = linux_rust_bindings::MAP_NONBLOCK;
        const MAP_STACK = linux_rust_bindings::MAP_STACK;
        const MAP_HUGETLB = linux_rust_bindings::MAP_HUGETLB;
        const MAP_SYNC = linux_rust_bindings::MAP_SYNC;
        const MAP_FIXED_NOREPLACE = linux_rust_bindings::MAP_FIXED_NOREPLACE;
        const MAP_UNINITIALIZED = linux_rust_bindings::MAP_UNINITIALIZED;
        const MAP_FILE = linux_rust_bindings::MAP_FILE;
        const MAP_GROWSDOWN = linux_rust_bindings::MAP_GROWSDOWN;
        const MAP_DENYWRITE = linux_rust_bindings::MAP_DENYWRITE;
        const MAP_EXECUTABLE = linux_rust_bindings::MAP_EXECUTABLE;
        const MAP_LOCKED = linux_rust_bindings::MAP_LOCKED;
        const MAP_NORESERVE = linux_rust_bindings::MAP_NORESERVE;
        const MAP_HUGE_SHIFT = linux_rust_bindings::MAP_HUGE_SHIFT;
        const MAP_HUGE_MASK = linux_rust_bindings::MAP_HUGE_MASK;
        const MAP_HUGE_16KB = linux_rust_bindings::MAP_HUGE_16KB;
        const MAP_HUGE_64KB = linux_rust_bindings::MAP_HUGE_64KB;
        const MAP_HUGE_512KB = linux_rust_bindings::MAP_HUGE_512KB;
        const MAP_HUGE_1MB = linux_rust_bindings::MAP_HUGE_1MB;
        const MAP_HUGE_2MB = linux_rust_bindings::MAP_HUGE_2MB;
        const MAP_HUGE_8MB = linux_rust_bindings::MAP_HUGE_8MB;
        const MAP_HUGE_16MB = linux_rust_bindings::MAP_HUGE_16MB;
        const MAP_HUGE_32MB = linux_rust_bindings::MAP_HUGE_32MB;
        const MAP_HUGE_256MB = linux_rust_bindings::MAP_HUGE_256MB;
        const MAP_HUGE_512MB = linux_rust_bindings::MAP_HUGE_512MB;
        const MAP_HUGE_1GB = linux_rust_bindings::MAP_HUGE_1GB;
        const MAP_HUGE_2GB = linux_rust_bindings::MAP_HUGE_2GB;
        const MAP_HUGE_16GB = linux_rust_bindings::MAP_HUGE_16GB as i32;
    }
}

/// Map files or devices into memory.
/// Almost impossible to make safe, and the [linux documentation](https://man7.org/linux/man-pages/man2/mmap.2.html)
/// should be consulted for details.
/// # Errors
/// See above
/// # Safety
/// see above
pub unsafe fn mmap(addr: Option<usize>, length: NonZeroUsize, memory_protection: MemoryProtection,
                   required_flag: MapRequiredFlag, additional_flags: MapAdditionalFlags,
                   fd: Option<Fd>, offset: OffT) -> crate::Result<usize> {
    let flags = required_flag.into_flag() | additional_flags;
    let res_ptr = syscall!(MMAP, addr.unwrap_or_default(), length.get(), memory_protection.bits(), flags.bits(), fd.unwrap_or(-1), offset);
    bail_on_below_zero!(res_ptr, "`MMAP` syscall failed");
    Ok(res_ptr)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_basic_alloc() {
        unsafe {
            let size = 4096;
            let sz = NonZeroUsize::new(size).unwrap();
            let stack = mmap(None, sz, MemoryProtection::PROT_READ | MemoryProtection::PROT_WRITE,
                             MapRequiredFlag::MapPrivate, MapAdditionalFlags::MAP_ANONYMOUS, None, 0).unwrap();
            let slice_stack: &mut [u8] = core::slice::from_raw_parts_mut(stack as _, size);
            for i in 0..slice_stack.len() {
                // The memory should be zeroed
                assert_eq!(0, slice_stack[i]);
                // The memory should be writeable
                slice_stack[i] = i as u8;
            }
            for i in 0..slice_stack.len() {
                assert_eq!(i as u8, slice_stack[i]);
            }
        }
    }
}