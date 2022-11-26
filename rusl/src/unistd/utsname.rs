use crate::string::strlen::buf_strlen;

// Not always true, see https://man7.org/linux/man-pages/man2/uname.2.html
pub const UTSNAME_LENGTH: usize = 65;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct UtsName {
    pub sysname: [u8; UTSNAME_LENGTH],
    pub nodename: [u8; UTSNAME_LENGTH],
    pub release: [u8; UTSNAME_LENGTH],
    pub version: [u8; UTSNAME_LENGTH],
    pub machine: [u8; UTSNAME_LENGTH],
}

macro_rules! get_name {
    ($path: ident) => {
        /// Will attempt to convert the buffered name to utf8.
        /// # Errors
        /// If the bytes are not valid utf8
        pub fn $path(&self) -> crate::Result<&str> {
            let len = buf_strlen(&self.$path)?;
            unsafe {
                // Safety:
                // We're already checking that the len is in bounds with buf_strlen
                Ok(core::str::from_utf8(&self.$path.get_unchecked(..len))?)
            }
        }
    };
}

impl UtsName {
    get_name!(sysname);
    get_name!(nodename);
    get_name!(release);
    get_name!(version);
    get_name!(machine);
}
