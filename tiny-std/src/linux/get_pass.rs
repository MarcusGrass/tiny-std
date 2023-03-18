use crate::error::Error;
use core::ops::{BitAndAssign, BitOrAssign};
use rusl::platform::{SetAction, STDIN};
use rusl::termios::{tcgetattr, tcsetattr};

/// Sets the terminal to no echo and waits for a line to be entered.
/// The raw input, including the newline, will be put into the buffer and converted to a &str.
/// If the buffer is too small to fit the input, an attempt will be made to drain stdin and then
/// return an error.
/// # Errors
/// Buffer undersized.
/// Stdin unreadable.
/// Fail to set terminal attributes.
pub fn get_pass(buf: &mut [u8]) -> crate::error::Result<&str> {
    /// If the user supplied a buffer that's too small we'd still like to drain stdin
    /// so that it doesn't leak.
    /// # Safety
    /// Safe if the buffer is not empty
    #[cold]
    unsafe fn drain_newline(buf: &mut [u8]) -> Result<(), Error> {
        loop {
            let res = rusl::unistd::read(STDIN, buf)?;
            // EOF or full line read
            if res == 0 || res < buf.len() {
                return Ok(());
            }
            let last = *buf.last().unwrap_unchecked();
            if last == b'\n' {
                return Ok(());
            }
        }
    }
    if buf.is_empty() {
        return Err(Error::Uncategorized(
            "Supplied a zero-length buffer to getpass, need an initialized buffer to populate",
        ));
    }
    let mut stdin_term = tcgetattr(STDIN)?;
    let orig_flags = stdin_term;
    let iflag = &mut stdin_term.0.c_lflag;
    iflag.bitand_assign(!(rusl::platform::ECHO as u32));
    iflag.bitor_assign(rusl::platform::ECHONL as u32);
    tcsetattr(STDIN, SetAction::NOW, &stdin_term)?;
    let read = rusl::unistd::read(STDIN, buf)?;
    unsafe {
        // Safety, we know the buffer is not empty.
        if read == buf.len() && *buf.last().unwrap_unchecked() != b'\n' {
            drain_newline(buf)?;
            tcsetattr(STDIN, SetAction::NOW, &orig_flags)?;
            return Err(Error::Uncategorized(
                "Supplied a buffer that was too small to getpass, buffer overrun.",
            ));
        }
    }
    tcsetattr(STDIN, SetAction::NOW, &orig_flags)?;
    core::str::from_utf8(&buf[..read])
        .map_err(|_| Error::no_code("Failed to convert read password to utf8.  "))
}
