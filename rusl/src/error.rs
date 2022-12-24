use core::fmt::{Display, Formatter};
use core::str::Utf8Error;

pub use crate::error::errno::Errno;

mod errno;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Copy, Clone)]
pub struct Error {
    pub msg: &'static str,
    pub code: Option<Errno>,
}

impl Error {
    pub(crate) const fn with_code(msg: &'static str, code: i32) -> Self {
        Self {
            msg,
            code: Some(Errno::new(code)),
        }
    }

    #[must_use]
    pub const fn no_code(msg: &'static str) -> Self {
        Self { msg, code: None }
    }

    #[cfg(not(feature = "alloc"))]
    #[must_use]
    pub const fn not_null_terminated() -> Self {
        Self {
            msg: "Path to be used as a null terminated string does not end with null and no alloc is available",
            code: None,
        }
    }
}

impl From<Utf8Error> for Error {
    fn from(_: Utf8Error) -> Self {
        Self {
            msg: "Failed to convert to utf8",
            code: None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        if let Some(code) = self.code {
            f.write_fmt(format_args!(
                "Error {{ msg: `{}`, errno: {code} }}",
                self.msg,
            ))
        } else {
            f.write_fmt(format_args!("Error {{ msg: `{}` }}", self.msg))
        }
    }
}

impl Display for Errno {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("[{}]: {}", self.raw(), self.as_str()))
    }
}
