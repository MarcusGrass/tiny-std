transparent_bitflags! {
    pub struct MemoryProtection: i32 {
        const DEFAULT = 0;
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
            MapRequiredFlag::MapShared => linux_rust_bindings::mman::MAP_SHARED as u32,
            MapRequiredFlag::MapSharedValidate => {
                linux_rust_bindings::mman::MAP_SHARED_VALIDATE as u32
            }
            MapRequiredFlag::MapPrivate => linux_rust_bindings::mman::MAP_PRIVATE as u32,
        })
    }
}

transparent_bitflags! {
    pub struct MapAdditionalFlags: u32 {
        const DEFAULT = 0;
        const MAP_TYPE = linux_rust_bindings::mman::MAP_TYPE as u32;
        const MAP_FIXED = linux_rust_bindings::mman::MAP_FIXED as u32;
        const MAP_ANONYMOUS = linux_rust_bindings::mman::MAP_ANONYMOUS as u32;
        const MAP_POPULATE = linux_rust_bindings::mman::MAP_POPULATE as u32;
        const MAP_NONBLOCK = linux_rust_bindings::mman::MAP_NONBLOCK as u32;
        const MAP_STACK = linux_rust_bindings::mman::MAP_STACK as u32;
        const MAP_HUGETLB = linux_rust_bindings::mman::MAP_HUGETLB as u32;
        const MAP_SYNC = linux_rust_bindings::mman::MAP_SYNC as u32;
        const MAP_FIXED_NOREPLACE = linux_rust_bindings::mman::MAP_FIXED_NOREPLACE as u32;
        const MAP_UNINITIALIZED = linux_rust_bindings::mman::MAP_UNINITIALIZED as u32;
        const MAP_FILE = linux_rust_bindings::mman::MAP_FILE as u32;
        const MAP_GROWSDOWN = linux_rust_bindings::mman::MAP_GROWSDOWN as u32;
        const MAP_DENYWRITE = linux_rust_bindings::mman::MAP_DENYWRITE as u32;
        const MAP_EXECUTABLE = linux_rust_bindings::mman::MAP_EXECUTABLE as u32;
        const MAP_LOCKED = linux_rust_bindings::mman::MAP_LOCKED as u32;
        const MAP_NORESERVE = linux_rust_bindings::mman::MAP_NORESERVE as u32;
        const MAP_HUGE_SHIFT = linux_rust_bindings::mman::MAP_HUGE_SHIFT as u32;
        const MAP_HUGE_MASK = linux_rust_bindings::mman::MAP_HUGE_MASK as u32;
        const MAP_HUGE_16KB = linux_rust_bindings::mman::MAP_HUGE_16KB as u32;
        const MAP_HUGE_64KB = linux_rust_bindings::mman::MAP_HUGE_64KB as u32;
        const MAP_HUGE_512KB = linux_rust_bindings::mman::MAP_HUGE_512KB as u32;
        const MAP_HUGE_1MB = linux_rust_bindings::mman::MAP_HUGE_1MB as u32;
        const MAP_HUGE_2MB = linux_rust_bindings::mman::MAP_HUGE_2MB as u32;
        const MAP_HUGE_8MB = linux_rust_bindings::mman::MAP_HUGE_8MB as u32;
        const MAP_HUGE_16MB = linux_rust_bindings::mman::MAP_HUGE_16MB as u32;
        const MAP_HUGE_32MB = linux_rust_bindings::mman::MAP_HUGE_32MB as u32;
        const MAP_HUGE_256MB = linux_rust_bindings::mman::MAP_HUGE_256MB as u32;
        const MAP_HUGE_512MB = linux_rust_bindings::mman::MAP_HUGE_512MB as u32;
        const MAP_HUGE_1GB = linux_rust_bindings::mman::MAP_HUGE_1GB as u32;
        const MAP_HUGE_2GB = linux_rust_bindings::mman::MAP_HUGE_2GB as u32;
        #[expect(clippy::cast_possible_truncation)]
        const MAP_HUGE_16GB = linux_rust_bindings::mman::MAP_HUGE_16GB as u32;
    }
}
