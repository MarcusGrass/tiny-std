// Not always true, see https://man7.org/linux/man-pages/man2/uname.2.html
pub const UTSNAME_LENGTH: usize = 65;

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct UtsName(linux_rust_bindings::utsname::new_utsname);

macro_rules! get_name {
    ($ident: ident) => {
        /// Will attempt to convert the buffered name to utf8.
        /// # Errors
        /// If the bytes are not valid utf8
        pub fn $ident(&self) -> crate::Result<&str> {
            unsafe {
                let as_u8: &[u8] = core::mem::transmute(self.0.$ident.as_slice());
                let len = crate::string::strlen::buf_strlen(as_u8)?;
                // Safety:
                // We're already checking that the len is in bounds with buf_strlen
                Ok(core::str::from_utf8(&as_u8.get_unchecked(..len))?)
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
