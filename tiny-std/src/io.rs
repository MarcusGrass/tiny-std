#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use core::{fmt, str};

use rusl::error::Errno;

use crate::error::{Error, Result};
use crate::io::read_buf::ReadBuf;

pub mod read_buf;

pub trait Read {
    /// Read into to provided buffer
    /// # Errors
    /// Eventual Errors specific to the implementation
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

    /// Reads to the end of this Reader,
    /// # Errors
    /// Eventual Errors specific to the implementation
    #[cfg(feature = "alloc")]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        default_read_to_end(self, buf)
    }

    /// Reads to the end of the provided buffer
    /// # Errors
    /// Eventual Errors specific to the implementation
    #[cfg(feature = "alloc")]
    fn read_to_string(&mut self, buf: &mut String) -> Result<usize> {
        default_read_to_string(self, buf)
    }

    /// Reads exactly enough bytes to fill the buffer
    /// # Errors
    /// Eventual Errors specific to the implementation
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        default_read_exact(self, buf)
    }

    /// Reads into the provided `ReadBuf`
    /// # Errors
    /// Eventual Errors specific to the implementation
    fn read_buf(&mut self, buf: &mut ReadBuf<'_>) -> Result<()> {
        default_read_buf(|b| self.read(b), buf)
    }

    /// Reads exactly enough bytes to fill the `ReadBuf`
    /// # Errors
    /// Eventual Errors specific to the implementation
    fn read_buf_exact(&mut self, buf: &mut ReadBuf<'_>) -> Result<()> {
        while buf.remaining() > 0 {
            let prev_filled = buf.filled().len();
            match self.read_buf(buf) {
                Ok(()) => {}
                Err(e) if e.matches_errno(Errno::EINTR) => continue,
                Err(e) => return Err(e),
            }

            if buf.filled().len() == prev_filled {
                return Err(Error::no_code("Failed to fill buffer"));
            }
        }

        Ok(())
    }

    /// Get this reader by mut ref
    /// # Errors
    /// Eventual Errors specific to the implementation
    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        self
    }
}

// This uses an adaptive system to extend the vector when it fills. We want to
// avoid paying to allocate and zero a huge chunk of memory if the reader only
// has 4 bytes while still making large reads if the reader does have a ton
// of data to return. Simply tacking on an extra DEFAULT_BUF_SIZE space every
// time is 4,500 times (!) slower than a default reservation size of 32 if the
// reader has a very small amount of data to return.
#[cfg(feature = "alloc")]
pub(crate) fn default_read_to_end<R: Read + ?Sized>(r: &mut R, buf: &mut Vec<u8>) -> Result<usize> {
    let start_len = buf.len();
    let start_cap = buf.capacity();

    let mut initialized = 0; // Extra initialized bytes from previous loop iteration
    loop {
        if buf.len() == buf.capacity() {
            buf.reserve(32); // buf is full, need more space
        }

        let mut read_buf = ReadBuf::uninit(buf.spare_capacity_mut());

        // SAFETY: These bytes were initialized but not filled in the previous loop
        unsafe {
            read_buf.assume_init(initialized);
        }

        match r.read_buf(&mut read_buf) {
            Ok(()) => {}
            Err(ref e) if e.matches_errno(Errno::EINTR) => continue,
            Err(e) => return Err(e),
        }

        if read_buf.filled_len() == 0 {
            return Ok(buf.len() - start_len);
        }

        // store how much was initialized but not filled
        initialized = read_buf.initialized_len() - read_buf.filled_len();
        let new_len = read_buf.filled_len() + buf.len();

        // SAFETY: ReadBuf's invariants mean this much memory is init
        unsafe {
            buf.set_len(new_len);
        }

        if buf.len() == buf.capacity() && buf.capacity() == start_cap {
            // The buffer might be an exact fit. Let's read into a probe buffer
            // and see if it returns `Ok(0)`. If so, we've avoided an
            // unnecessary doubling of the capacity. But if not, append the
            // probe buffer to the primary buffer and let its capacity grow.
            let mut probe = [0u8; 32];

            loop {
                match r.read(&mut probe) {
                    Ok(0) => return Ok(buf.len() - start_len),
                    Ok(n) => {
                        buf.extend_from_slice(&probe[..n]);
                        break;
                    }
                    Err(ref e) if e.matches_errno(Errno::EINTR) => continue,
                    Err(e) => return Err(e),
                }
            }
        }
    }
}

#[cfg(feature = "alloc")]
pub(crate) fn default_read_to_string<R: Read + ?Sized>(
    r: &mut R,
    buf: &mut String,
) -> Result<usize> {
    // Note that we do *not* call `r.read_to_end()` here. We are passing
    // `&mut Vec<u8>` (the raw contents of `buf`) into the `read_to_end`
    // method to fill it up. An arbitrary implementation could overwrite the
    // entire contents of the vector, not just append to it (which is what
    // we are expecting).
    //
    // To prevent extraneously checking the UTF-8-ness of the entire buffer
    // we pass it to our hardcoded `default_read_to_end` implementation which
    // we know is guaranteed to only read data into the end of the buffer.
    unsafe { append_to_string(buf, |b| default_read_to_end(r, b)) }
}

// Several `read_to_string` and `read_line` methods in the standard library will
// append data into a `String` buffer, but we need to be pretty careful when
// doing this. The implementation will just call `.as_mut_vec()` and then
// delegate to a byte-oriented reading method, but we must ensure that when
// returning we never leave `buf` in a state such that it contains invalid UTF-8
// in its bounds.
//
// To this end, we use an RAII guard (to protect against panics) which updates
// the length of the string when it is dropped. This guard initially truncates
// the string to the prior length and only after we've validated that the
// new contents are valid UTF-8 do we allow it to set a longer length.
//
// The unsafety in this function is twofold:
//
// 1. We're looking at the raw bytes of `buf`, so we take on the burden of UTF-8
//    checks.
// 2. We're passing a raw buffer to the function `f`, and it is expected that
//    the function only *appends* bytes to the buffer. We'll get undefined
//    behavior if existing bytes are overwritten to have non-UTF-8 data.
#[cfg(feature = "alloc")]
pub(crate) unsafe fn append_to_string<F>(buf: &mut String, f: F) -> Result<usize>
where
    F: FnOnce(&mut Vec<u8>) -> Result<usize>,
{
    struct Guard<'a> {
        buf: &'a mut Vec<u8>,
        len: usize,
    }

    impl Drop for Guard<'_> {
        fn drop(&mut self) {
            unsafe {
                self.buf.set_len(self.len);
            }
        }
    }

    let mut g = Guard {
        len: buf.len(),
        buf: buf.as_mut_vec(),
    };
    let ret = f(g.buf);
    if str::from_utf8(&g.buf[g.len..]).is_err() {
        ret.and_then(|_| Err(Error::no_code("Stream did not contain valid UTF-8")))
    } else {
        g.len = g.buf.len();
        ret
    }
}

pub(crate) fn default_read_exact<R: Read + ?Sized>(this: &mut R, mut buf: &mut [u8]) -> Result<()> {
    while !buf.is_empty() {
        match this.read(buf) {
            Ok(0) => break,
            Ok(n) => {
                let tmp = buf;
                buf = &mut tmp[n..];
            }
            Err(ref e) if e.matches_errno(Errno::EINTR) => {}
            Err(e) => return Err(e),
        }
    }
    if buf.is_empty() {
        Ok(())
    } else {
        Err(Error::no_code("Failed to fill whole buffer"))
    }
}

pub(crate) fn default_read_buf<F>(read: F, buf: &mut ReadBuf<'_>) -> Result<()>
where
    F: FnOnce(&mut [u8]) -> Result<usize>,
{
    let n = read(buf.initialize_unfilled())?;
    buf.add_filled(n);
    Ok(())
}

pub trait Write {
    /// Tries to write the contents of the provided buffer into this writer
    /// returning how many bytes were written.
    /// # Errors
    /// `Writer` failing to write
    fn write(&mut self, buf: &[u8]) -> Result<usize>;

    /// Flushes this `Writer`
    /// # Errors
    /// Formatting the provided arguments, or the `Writer` failing to write
    fn flush(&mut self) -> Result<()>;

    /// Writes the full buffer into this `Writer`
    /// # Errors
    /// `Writer` failing to write
    fn write_all(&mut self, mut buf: &[u8]) -> Result<()> {
        while !buf.is_empty() {
            match self.write(buf) {
                Ok(0) => {
                    return Err(Error::no_code("failed to write whole buffer"));
                }
                Ok(n) => buf = &buf[n..],
                Err(ref e) if e.matches_errno(Errno::EINTR) => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    /// Writes format arguments into this `Writer`
    /// # Errors
    /// Formatting the provided arguments, or the `Writer` failing to write
    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> Result<()> {
        // Create a shim which translates a Write to a fmt::Write and saves
        // off I/O errors. instead of discarding them
        struct Adapter<'a, T: ?Sized + 'a> {
            inner: &'a mut T,
            error: Result<()>,
        }

        impl<T: Write + ?Sized> fmt::Write for Adapter<'_, T> {
            fn write_str(&mut self, s: &str) -> fmt::Result {
                match self.inner.write_all(s.as_bytes()) {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        self.error = Err(e);
                        Err(fmt::Error)
                    }
                }
            }
        }

        let mut output = Adapter {
            inner: self,
            error: Ok(()),
        };
        match fmt::write(&mut output, fmt) {
            Ok(()) => Ok(()),
            Err(..) => {
                // check if the error came from the underlying `Write` or not
                if output.error.is_err() {
                    output.error
                } else {
                    Err(Error::no_code("formatter error"))
                }
            }
        }
    }

    /// Get this `Writer` as a mutable reference
    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        self
    }
}
