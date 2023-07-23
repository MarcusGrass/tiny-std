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
pub fn buf_strlen(buf: &[u8]) -> Result<usize, Error> {
    for (ind, byte) in buf.iter().enumerate() {
        if *byte == 0 {
            return Ok(ind);
        }
    }
    Err(Error::no_code("String isn't null terminated"))
}
