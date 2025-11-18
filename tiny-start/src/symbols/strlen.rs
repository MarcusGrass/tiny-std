#[no_mangle]
#[inline(always)]
#[expect(clippy::missing_safety_doc)]
// 64-bit linux which is the only target supported
// has a 64-bit long int, this needs to be updated
// if 32-bit support is needed
#[cfg(target_pointer_width = "64")]
/// Heavily inspired by glibc's implementation
pub unsafe extern "C" fn strlen(s: *const u8) -> usize
{
    const HIMAGIC: usize = 0x80808080;
    const LOMAGIC: usize = 0x01010101;
    const EXIT_MASK: usize = core::mem::size_of::<usize>() - 1;

    let mut char_ptr = s;
    while (char_ptr.addr() & EXIT_MASK) != 0 {
        if char_ptr.read() == b'\0' {
            return char_ptr.addr() - s.addr();
        }
        char_ptr = char_ptr.add( 1);
    }
    let mut longword_ptr = char_ptr as *const usize;
    loop {
        let longword = longword_ptr.read();
        if ((longword - LOMAGIC) & HIMAGIC) != 0 {
            let ch_ptr = longword_ptr as *const u8;
            for i in 0..core::mem::size_of::<usize>() {
                if ch_ptr.add(i).read() == 0 {
                    return ch_ptr.addr() - s.addr() + i;
                }
            }
        }
        longword_ptr = longword_ptr.add(1);
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
        const TEST_STR: &[u8; 82] = b"hello, this is a fairly long string, at least long enough to cover a u64 boundary\0";
        unsafe {
            assert_eq!(81, super::strlen(TEST_STR.as_ptr()))
        }
    }
}
