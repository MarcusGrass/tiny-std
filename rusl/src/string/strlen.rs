use crate::Error;

/// Gets the length of a null terminated string
/// # Safety
/// Safe only if the pointer is null terminated
#[must_use]
pub const unsafe fn strlen(s: *const u8) -> usize {
    let mut i = 0;
    loop {
        if s.add(i).read() == 0 {
            return i;
        }
        i += 1;
    }
}

/// Gets the length up until null termination for this buffer
/// # Errors
/// Buffer isn't null terminated
pub const fn buf_strlen(buf: &[u8]) -> Result<usize, Error> {
    let mut ind = 0;
    while ind < buf.len() {
        if buf[ind] == 0 {
            return Ok(ind);
        }
        ind += 1;
    }
    Err(Error::no_code("String isn't null terminated"))
}
