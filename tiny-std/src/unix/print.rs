//! Synchronized printing to stdout and stderr
use crate::sync::Mutex;

pub static __STDOUT_LOCK: Mutex<()> = Mutex::new(());
pub static __STDERR_LOCK: Mutex<()> = Mutex::new(());

use rusl::platform::Fd;

/// Corresponds to std's `print!`-macro
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        {
            let __tiny_std_unix_print_writer_guard = $crate::unix::print::__STDOUT_LOCK.lock();
            let mut __tiny_std_unix_print_writer = $crate::unix::print::__STDOUT_WRITER;
            let _ = core::fmt::Write::write_fmt(&mut __tiny_std_unix_print_writer, format_args!($($arg)*));
        }
    }
}

/// Corresponds to std's `println!`-macro
#[macro_export]
macro_rules! println {
    () => {
        {
            let __tiny_std_unix_print_writer_guard = $crate::unix::print::__STDOUT_LOCK.lock();
            let __tiny_std_unix_print_writer = $crate::unix::print::__STDOUT_WRITER;
            let _ = __tiny_std_unix_print_writer.__write_newline();
        }
    };
    ($($arg:tt)*) => {
        {
            let __tiny_std_unix_print_writer_guard = $crate::unix::print::__STDOUT_LOCK.lock();
            let mut __tiny_std_unix_print_writer = $crate::unix::print::__STDOUT_WRITER;
            let _ = core::fmt::Write::write_fmt(&mut __tiny_std_unix_print_writer, format_args!($($arg)*));
            let _ = __tiny_std_unix_print_writer.__write_newline();
        }
    }
}

/// Corresponds to std's `eprint!`-macro
#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => {
        {
            let __tiny_std_unix_print_writer_guard = $crate::unix::print::__STDERR_LOCK.lock();
            let mut __tiny_std_unix_print_writer = $crate::unix::print::__STDERR_WRITER;
            let _ = core::fmt::Write::write_fmt(&mut __tiny_std_unix_print_writer, format_args!($($arg)*));
        }
    }
}

/// Corresponds to std's `eprintln!`-macro
#[macro_export]
macro_rules! eprintln {
    () => {
        {
            let __tiny_std_unix_print_writer_guard = $crate::unix::print::__STDERR_LOCK.lock();
            let __tiny_std_unix_print_writer = $crate::unix::print::__STDERR_WRITER;
            let _ = __tiny_std_unix_print_writer.__write_newline();
        }
    };
    ($($arg:tt)*) => {
        {
            let __tiny_std_unix_print_writer_guard = $crate::unix::print::__STDERR_LOCK.lock();
            let mut __tiny_std_unix_print_writer = $crate::unix::print::__STDERR_WRITER;
            let _ = core::fmt::Write::write_fmt(&mut __tiny_std_unix_print_writer, format_args!($($arg)*));
            let _ = __tiny_std_unix_print_writer.__write_newline();
        }
    }
}

/// Corresponds to std's `dbg!`-macro
#[macro_export]
macro_rules! dbg {
    () => {
        $crate::eprintln!("[{}:{}]", core::file!(), core::line!())
    };
    ($val:expr $(,)?) => {
        match $val {
            tmp => {
                $crate::eprintln!("[{}:{}] {} = {:#?}",
                    core::file!(), core::line!(), core::stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}

fn try_print(fd: Fd, msg: &str) -> core::fmt::Result {
    let buf = msg.as_bytes();
    let len = buf.len();
    let mut flushed = 0;
    loop {
        let res = rusl::unistd::write(fd, &buf[flushed..]).map_err(|_e| core::fmt::Error)?;
        match res.cmp(&0) {
            core::cmp::Ordering::Less => return Err(core::fmt::Error),
            core::cmp::Ordering::Equal => return Ok(()),
            core::cmp::Ordering::Greater => {
                // Greater than zero
                flushed += res as usize;
                if flushed >= len {
                    return Ok(());
                }
            }
        }
    }
}

pub struct __UnixWriter(Fd);

pub const __STDOUT_WRITER: __UnixWriter = __UnixWriter(rusl::platform::STDOUT);
pub const __STDERR_WRITER: __UnixWriter = __UnixWriter(rusl::platform::STDERR);
impl __UnixWriter {
    /// # Errors
    /// Will return an error if the underlying syscall fails
    pub fn __write_newline(&self) -> core::fmt::Result {
        try_print(self.0, "\n")
    }
}

impl core::fmt::Write for __UnixWriter {
    #[inline]
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        try_print(self.0, s)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_prints() {
        print!("-- First");
        print!("My first");
        print!(" two messages");
        print!(" were cutoff but it's fine");
        println!();
        println!("-- Second\nHello there {}", "me");
    }

    #[test]
    fn test_eprints() {
        eprintln!("-- First");
        eprint!("My first");
        eprint!(" two messages");
        eprint!(" were cutoff but it's fine");
        eprintln!();
        eprintln!("-- Second\nHello there {}", "me");
    }

    #[test]
    fn test_dbgs() {
        dbg!();
        let val = 5;
        let res = dbg!(val) - 5;
        assert_eq!(0, res);
    }
}
