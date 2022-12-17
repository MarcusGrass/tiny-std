/// Mode accepted by the [open syscall](https://man7.org/linux/man-pages/man2/open.2.html)
transparent_bitflags! {
    pub struct Mode: u32 {
        const S_IRWXU = linux_rust_bindings::stat::S_IRWXU as u32; // 00700 user read write exec
        const S_IRUSR = linux_rust_bindings::stat::S_IRUSR as u32; // 00400 user Read
        const S_IWUSR = linux_rust_bindings::stat::S_IWUSR as u32; // 00200 user write
        const S_IXUSR = linux_rust_bindings::stat::S_IXUSR as u32;  // 00100 user execute
        const S_IRWXG = linux_rust_bindings::stat::S_IRWXG as u32;  // 00070 group read write exec
        const S_IRGRP = linux_rust_bindings::stat::S_IRGRP as u32;  // 00040 group read
        const S_IWGRP = linux_rust_bindings::stat::S_IWGRP as u32;  // 00020 group write
        const S_IXGRP = linux_rust_bindings::stat::S_IXGRP as u32;   // 00010 group exec
        const S_IRWXO = linux_rust_bindings::stat::S_IRWXO as u32;   // 00007 other read write exec
        const S_IROTH = linux_rust_bindings::stat::S_IROTH as u32;   // 00004 other read
        const S_IWOTH = linux_rust_bindings::stat::S_IWOTH as u32;   // 00002 other write
        const S_IXOTH = linux_rust_bindings::stat::S_IXOTH as u32;   // 00001 other execute

        // Linux specific bits
        const S_ISUID = linux_rust_bindings::stat::S_ISUID as u32; // 0004000 set-user-ID bit
        const S_ISGID = linux_rust_bindings::stat::S_ISGID as u32; // 0002000 set-group-ID bit
        const S_ISVTX = linux_rust_bindings::stat::S_ISVTX as u32; // 0001000 set-sticky bit

        // File specific bits
        const S_IFIFO  = linux_rust_bindings::stat::S_IFIFO as u32;
        const S_IFCHR  = linux_rust_bindings::stat::S_IFCHR as u32;
        const S_IFDIR  = linux_rust_bindings::stat::S_IFDIR as u32;
        const S_IFBLK  = linux_rust_bindings::stat::S_IFBLK as u32;
        const S_IFREG  = linux_rust_bindings::stat::S_IFREG as u32;
        const S_IFLNK  = linux_rust_bindings::stat::S_IFLNK as u32;
        const S_IFSOCK = linux_rust_bindings::stat::S_IFSOCK as u32;
        const S_IFMT   = linux_rust_bindings::stat::S_IFMT as u32;
    }
}
