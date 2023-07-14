use crate::fs::File;
use crate::io::Read;
use rusl::error::Errno;

/// Fills the provided buffer with random bytes from /dev/random.
/// # Errors
/// File permissions failure
pub fn system_random(buf: &mut [u8]) -> crate::error::Result<()> {
    let mut file = File::open("/dev/random\0")?;
    let mut offset = 0;
    while offset < buf.len() {
        match file.read(&mut buf[offset..]) {
            Ok(read) => {
                offset += read;
            }
            Err(e) => {
                if e.matches_errno(Errno::EINTR) {
                    continue;
                }
                return Err(e);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gets_random() {
        let mut buf = [0u8; 4096];
        system_random(&mut buf).unwrap();
        // Likely outcome is 16 around zeroes
        let mut count_zero = 0;
        for i in buf {
            if i == 0 {
                count_zero += 1;
            }
        }
        assert!(count_zero < 32, "After filling a buf with random bytes {count_zero} zeroes were found, should be around 16.");
    }
}
