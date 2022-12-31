use core::fmt::{Debug, Display, Formatter};

pub use rusl::error::Errno;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Copy, Clone)]
pub enum Error {
    Uncategorized(&'static str),
    Os { msg: &'static str, code: Errno },
}

impl Error {
    #[inline]
    pub(crate) const fn no_code(msg: &'static str) -> Self {
        Self::Uncategorized(msg)
    }

    #[inline]
    pub(crate) const fn os(msg: &'static str, code: Errno) -> Self {
        Self::Os { msg, code }
    }

    #[inline]
    #[must_use]
    pub fn matches_errno(&self, errno: Errno) -> bool {
        if let Self::Os { code, .. } = self {
            *code == errno
        } else {
            false
        }
    }
}

impl From<rusl::Error> for Error {
    fn from(e: rusl::Error) -> Self {
        if let Some(code) = e.code {
            Self::Os { msg: e.msg, code }
        } else {
            Self::Uncategorized(e.msg)
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::Uncategorized(msg) => f.write_fmt(format_args!("Error: {msg}")),
            Error::Os { msg, code } => {
                f.write_fmt(format_args!("OsError {{ msg: `{msg}`, code: {code} }}"))
            }
        }
    }
}
