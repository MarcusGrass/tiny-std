transparent_bitflags! {
    pub struct MemoryProtection: i32 {
        const PROT_NONE = linux_rust_bindings::mman::PROT_NONE;
        const PROT_READ = linux_rust_bindings::mman::PROT_READ;
        const PROT_WRITE = linux_rust_bindings::mman::PROT_WRITE;
        const PROT_EXEC = linux_rust_bindings::mman::PROT_EXEC;
        const PROT_SEM = linux_rust_bindings::mman::PROT_SEM;
        const PROT_GROWSDOWN = linux_rust_bindings::mman::PROT_GROWSDOWN;
        const PROT_GROWSUP = linux_rust_bindings::mman::PROT_GROWSUP;
    }
}

#[derive(Debug, Copy, Clone)]
pub enum MapRequiredFlag {
    MapShared,
    MapSharedValidate,
    MapPrivate,
}

impl MapRequiredFlag {
    pub(crate) const fn into_flag(self) -> MapAdditionalFlags {
        MapAdditionalFlags(match self {
            MapRequiredFlag::MapShared => linux_rust_bindings::mman::MAP_SHARED,
            MapRequiredFlag::MapSharedValidate => linux_rust_bindings::mman::MAP_SHARED_VALIDATE,
            MapRequiredFlag::MapPrivate => linux_rust_bindings::mman::MAP_PRIVATE,
        })
    }
}

transparent_bitflags! {
    pub struct MapAdditionalFlags: i32 {
        const MAP_TYPE = linux_rust_bindings::mman::MAP_TYPE;
        const MAP_FIXED = linux_rust_bindings::mman::MAP_FIXED;
        const MAP_ANONYMOUS = linux_rust_bindings::mman::MAP_ANONYMOUS;
        const MAP_POPULATE = linux_rust_bindings::mman::MAP_POPULATE;
        const MAP_NONBLOCK = linux_rust_bindings::mman::MAP_NONBLOCK;
        const MAP_STACK = linux_rust_bindings::mman::MAP_STACK;
        const MAP_HUGETLB = linux_rust_bindings::mman::MAP_HUGETLB;
        const MAP_SYNC = linux_rust_bindings::mman::MAP_SYNC;
        const MAP_FIXED_NOREPLACE = linux_rust_bindings::mman::MAP_FIXED_NOREPLACE;
        const MAP_UNINITIALIZED = linux_rust_bindings::mman::MAP_UNINITIALIZED;
        const MAP_FILE = linux_rust_bindings::mman::MAP_FILE;
        const MAP_GROWSDOWN = linux_rust_bindings::mman::MAP_GROWSDOWN;
        const MAP_DENYWRITE = linux_rust_bindings::mman::MAP_DENYWRITE;
        const MAP_EXECUTABLE = linux_rust_bindings::mman::MAP_EXECUTABLE;
        const MAP_LOCKED = linux_rust_bindings::mman::MAP_LOCKED;
        const MAP_NORESERVE = linux_rust_bindings::mman::MAP_NORESERVE;
        const MAP_HUGE_SHIFT = linux_rust_bindings::mman::MAP_HUGE_SHIFT;
        const MAP_HUGE_MASK = linux_rust_bindings::mman::MAP_HUGE_MASK;
        const MAP_HUGE_16KB = linux_rust_bindings::mman::MAP_HUGE_16KB;
        const MAP_HUGE_64KB = linux_rust_bindings::mman::MAP_HUGE_64KB;
        const MAP_HUGE_512KB = linux_rust_bindings::mman::MAP_HUGE_512KB;
        const MAP_HUGE_1MB = linux_rust_bindings::mman::MAP_HUGE_1MB;
        const MAP_HUGE_2MB = linux_rust_bindings::mman::MAP_HUGE_2MB;
        const MAP_HUGE_8MB = linux_rust_bindings::mman::MAP_HUGE_8MB;
        const MAP_HUGE_16MB = linux_rust_bindings::mman::MAP_HUGE_16MB;
        const MAP_HUGE_32MB = linux_rust_bindings::mman::MAP_HUGE_32MB;
        const MAP_HUGE_256MB = linux_rust_bindings::mman::MAP_HUGE_256MB;
        const MAP_HUGE_512MB = linux_rust_bindings::mman::MAP_HUGE_512MB;
        const MAP_HUGE_1GB = linux_rust_bindings::mman::MAP_HUGE_1GB;
        const MAP_HUGE_2GB = linux_rust_bindings::mman::MAP_HUGE_2GB;
        const MAP_HUGE_16GB = linux_rust_bindings::mman::MAP_HUGE_16GB as i32;
    }
}
