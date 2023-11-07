use core::fmt::{Arguments, Debug, Display, Formatter, Write};
use rusl::string::unix_str::UnixStr;

/// Parses provided args of this running executable into the provided struct
/// On failure, prints help, then exits 1
pub fn parse_cli_args<T: ArgParse>() -> T {
    let mut args_os = crate::env::args_os();
    // Pop off this bin
    let _bin = args_os.next();
    match T::arg_parse(&mut args_os) {
        Ok(v) => v,
        Err(e) => {
            crate::eprintln!("{}", e);
            crate::process::exit(1);
        }
    }
}

pub trait ArgParse: Sized {
    type HelpPrinter: Display + Sized;
    fn arg_parse(args: &mut impl Iterator<Item = &'static UnixStr>) -> Result<Self, ArgParseError>;

    fn help_printer() -> &'static Self::HelpPrinter;
}

pub trait SubcommandParse: Sized {
    /// Helper struct for no-alloc printing
    type HelpPrinter: Display + Sized;

    fn help_printer() -> &'static Self::HelpPrinter;

    fn subcommand_parse(
        cmd: &'static UnixStr,
        args: &mut impl Iterator<Item = &'static UnixStr>,
    ) -> Result<Option<Self>, ArgParseError>;
}

#[derive(Clone)]
pub struct ArgParseError {
    pub relevant_help: &'static dyn Display,
    pub cause: ArgParseCauseBuffer,
}

impl ArgParseError {
    const OVERFLOW_MSG: [u8; STACK_BUFFER_CAP] = *b"Cause unknown, too many characters to write into output buffer (BUG)\0\0\0\0\0\0\
    \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
    const OVERFLOW_BUF: ArgParseCauseBuffer = ArgParseCauseBuffer {
        buf: Self::OVERFLOW_MSG,
        len: 68,
    };
    pub fn new_cause_str(
        relevant_help: &'static dyn Display,
        cause: &str,
    ) -> Result<Self, ArgParseError> {
        let mut buf = ArgParseCauseBuffer::new();
        buf.write_str(cause).map_err(|_e| ArgParseError {
            relevant_help,
            cause: Self::OVERFLOW_BUF,
        })?;
        Ok(Self {
            relevant_help,
            cause: buf,
        })
    }

    pub fn new_cause_fmt(
        relevant_help: &'static dyn Display,
        cause: Arguments<'_>,
    ) -> Result<Self, ArgParseError> {
        let mut buf = ArgParseCauseBuffer::new();
        buf.oneshot_write(cause).map_err(|_e| ArgParseError {
            relevant_help,
            cause: Self::OVERFLOW_BUF,
        })?;
        Ok(Self {
            relevant_help,
            cause: buf,
        })
    }
}

impl Debug for ArgParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!(
            "ArgParseError {{ relevant_help: {}, cause: {}}}",
            self.relevant_help, self.cause
        ))
    }
}

impl Display for ArgParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{}{}", self.relevant_help, self.cause))
    }
}

const STACK_BUFFER_CAP: usize = 128;
#[derive(Debug, Copy, Clone)]
pub struct ArgParseCauseBuffer {
    buf: [u8; STACK_BUFFER_CAP],
    len: usize,
}

impl Write for ArgParseCauseBuffer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        if self.len != 0 {
            return Err(core::fmt::Error::default());
        }
        let buf_write = s.as_bytes();
        let rem = STACK_BUFFER_CAP - self.len;
        if buf_write.len() > rem {
            return Err(core::fmt::Error::default());
        }
        self.buf
            .get_mut(self.len..self.len + buf_write.len())
            .unwrap()
            .copy_from_slice(buf_write);
        self.len += buf_write.len();
        Ok(())
    }
}

impl ArgParseCauseBuffer {
    const fn new() -> Self {
        Self {
            buf: [0u8; STACK_BUFFER_CAP],
            len: 0,
        }
    }

    /// Can write into this buffer at most once
    #[inline]
    fn oneshot_write(&mut self, args: Arguments<'_>) -> core::fmt::Result {
        Self::write_fmt(self, args)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
}

impl Display for ArgParseCauseBuffer {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let s = core::str::from_utf8(&self.buf[..self.len])
            .map_err(|_e| core::fmt::Error::default())?;
        f.write_str(s)
    }
}
