use crate::string::unix_str::UnixStr;

#[derive(Debug, Copy, Clone)]
pub struct FilesystemType(pub(crate) *const u8);

unsafe impl Send for FilesystemType {}
unsafe impl Sync for FilesystemType {}

// Incomplete list
impl FilesystemType {
    pub const EXT4: Self = Self(UnixStr::from_str_checked("ext4\0").0.as_ptr());
    pub const TMPFS: Self = Self(UnixStr::from_str_checked("tmpfs\0").0.as_ptr());
    pub const DEVTMPFS: Self = Self(UnixStr::from_str_checked("devtmpfs\0").0.as_ptr());
    pub const SYSFS: Self = Self(UnixStr::from_str_checked("sysfs\0").0.as_ptr());
    pub const PROC: Self = Self(UnixStr::from_str_checked("proc\0").0.as_ptr());
    pub const VFAT: Self = Self(UnixStr::from_str_checked("vfat\0").0.as_ptr());
}

transparent_bitflags! {
    pub struct Mountflags: u64 {
        const DEFAULT = 0;
        const MS_RDONLY = linux_rust_bindings::mount::MS_RDONLY as u64;
        const MS_NOSUID = linux_rust_bindings::mount::MS_NOSUID as u64;
        const MS_NODEV = linux_rust_bindings::mount::MS_NODEV as u64;
        const MS_NOEXEC = linux_rust_bindings::mount::MS_NOEXEC as u64;
        const MS_SYNCHRONOUS = linux_rust_bindings::mount::MS_SYNCHRONOUS as u64;
        const MS_REMOUNT = linux_rust_bindings::mount::MS_REMOUNT as u64;
        const MS_MANDLOCK = linux_rust_bindings::mount::MS_MANDLOCK as u64;
        const MS_DIRSYNC = linux_rust_bindings::mount::MS_DIRSYNC as u64;
        const MS_NOSYMFOLLOW = linux_rust_bindings::mount::MS_NOSYMFOLLOW as u64;
        const MS_NOATIME = linux_rust_bindings::mount::MS_NOATIME as u64;
        const MS_NODIRATIME = linux_rust_bindings::mount::MS_NODIRATIME as u64;
        const MS_BIND = linux_rust_bindings::mount::MS_BIND as u64;
        const MS_MOVE = linux_rust_bindings::mount::MS_MOVE as u64;
        const MS_REC = linux_rust_bindings::mount::MS_REC as u64;
        const MS_VERBOSE = linux_rust_bindings::mount::MS_VERBOSE as u64;
        const MS_SILENT = linux_rust_bindings::mount::MS_SILENT as u64;
        const MS_POSIXACL = linux_rust_bindings::mount::MS_POSIXACL as u64;
        const MS_UNBINDABLE = linux_rust_bindings::mount::MS_UNBINDABLE as u64;
        const MS_PRIVATE = linux_rust_bindings::mount::MS_PRIVATE as u64;
        const MS_SLAVE = linux_rust_bindings::mount::MS_SLAVE as u64;
        const MS_SHARED = linux_rust_bindings::mount::MS_SHARED as u64;
        const MS_RELATIME = linux_rust_bindings::mount::MS_RELATIME as u64;
        const MS_KERNMOUNT = linux_rust_bindings::mount::MS_KERNMOUNT as u64;
        const MS_I_VERSION = linux_rust_bindings::mount::MS_I_VERSION as u64;
        const MS_STRICTATIME = linux_rust_bindings::mount::MS_STRICTATIME as u64;
        const MS_LAZYTIME = linux_rust_bindings::mount::MS_LAZYTIME as u64;
        const MS_SUBMOUNT = linux_rust_bindings::mount::MS_SUBMOUNT as u64;
        const MS_NOREMOTELOCK = linux_rust_bindings::mount::MS_NOREMOTELOCK as u64;
        const MS_NOSEC = linux_rust_bindings::mount::MS_NOSEC as u64;
        const MS_BORN = linux_rust_bindings::mount::MS_BORN as u64;
        const MS_ACTIVE = linux_rust_bindings::mount::MS_ACTIVE as u64;
        const MS_NOUSER = linux_rust_bindings::mount::MS_NOUSER as u64;
        const MS_RMT_MASK = linux_rust_bindings::mount::MS_RMT_MASK as u64;
        const MS_MGC_VAL = linux_rust_bindings::mount::MS_MGC_VAL as u64;
        const MS_MGC_MSK = linux_rust_bindings::mount::MS_MGC_MSK as u64;
    }
}
