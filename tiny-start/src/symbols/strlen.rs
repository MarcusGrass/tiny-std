#[no_mangle]
#[expect(clippy::missing_safety_doc)]
// 64-bit linux which is the only target supported
// has a 64-bit long int, this needs to be updated
// if 32-bit support is needed
#[cfg(target_pointer_width = "64")]
pub unsafe extern "C" fn strlen(s: *const u8) -> usize {
    let mut p = s;
    loop {
        if p.read() == 0 {
            return p.addr() - s.addr();
        }
        p = p.add(1);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn strlen_empty() {
        unsafe {
            let ptr = b"\0".as_ptr();
            assert_eq!(0, super::strlen(ptr))
        }
    }

    #[test]
    fn strlen_long() {
        const TEST_STR: &[u8; 82] =
            b"hello, this is a fairly long string, at least long enough to cover a u64 boundary\0";
        unsafe { assert_eq!(81, super::strlen(TEST_STR.as_ptr())) }
    }
}
