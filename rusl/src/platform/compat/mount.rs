use crate::string::unix_str::UnixStr;

#[derive(Debug, Copy, Clone)]
pub enum FilesystemType {
    Ext4,
    Tmpfs,
    Devtmpfs,
    Sysfs,
    Proc,
    Vfat,
}

impl FilesystemType {
    pub(crate) fn label(self) -> &'static UnixStr {
        unsafe {
            match self {
                FilesystemType::Ext4 => UnixStr::from_str_unchecked("ext4\0"),
                FilesystemType::Tmpfs => UnixStr::from_str_unchecked("tmpfs\0"),
                FilesystemType::Devtmpfs => UnixStr::from_str_unchecked("devtmpfs\0"),
                FilesystemType::Sysfs => UnixStr::from_str_unchecked("sysfs\0"),
                FilesystemType::Proc => UnixStr::from_str_unchecked("proc\0"),
                FilesystemType::Vfat => UnixStr::from_str_unchecked("vfat\0"),
            }
        }
    }
}
