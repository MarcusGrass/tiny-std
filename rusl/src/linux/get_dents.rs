use sc::syscall;

use crate::platform::{Fd, InoT, OffT, NULL_BYTE};

/// Reads directory entities into the provided buffer and returns the number of bytes read
/// See [Linux docs for details](https://man7.org/linux/man-pages/man2/getdents.2.html)
/// # Errors
/// See above
pub fn get_dents(fd: Fd, dir_p: &mut [u8]) -> crate::Result<usize> {
    let res = unsafe { syscall!(GETDENTS64, fd, dir_p.as_mut_ptr(), dir_p.len()) };
    bail_on_below_zero!(res, "`GETDENTS64` syscall failed");
    Ok(res)
}

#[derive(Copy, Clone)]
pub struct Dirent {
    pub d_ino: InoT,
    pub d_off: OffT,
    pub d_reclen: u16,
    pub d_type: u8,
    pub d_name: [u8; 256],
    // <- One padding byte here
}

impl Dirent {
    const LEN_OFFSET: usize = core::mem::size_of::<InoT>() + core::mem::size_of::<OffT>();
    const HEADER_SIZE: usize = Self::LEN_OFFSET + 2;
    const INOT_SIZE: usize = core::mem::size_of::<InoT>();
    const OFFT_SIZE: usize = core::mem::size_of::<OffT>();
    const NAME_START: usize = Self::HEADER_SIZE + 1;

    /// Try to parse a Dirent from the memory inside of a buffer filled by the `GETDENTS64` syscall
    /// # Safety
    /// Essentially only safe if used specifically on a buffer filled by `get_dents` starting with
    /// a valid offset which would be `0 + n1 + ... + nm` where n is the `d_reclen`
    /// of the `m`th already parsed `Dirent` from that same buffer.
    #[inline]
    #[must_use]
    pub unsafe fn try_from_bytes(buf: &[u8]) -> Option<Self> {
        let header = buf.get(0..Self::HEADER_SIZE)?;
        // Could get_unchecked
        let len = {
            // Safety: We just checked it's in range, transmute the two bytes into a single ne-u16, get it
            let len_buf = header.get_unchecked(Self::LEN_OFFSET..Self::HEADER_SIZE);
            *core::mem::transmute::<&[u8], &[u16]>(len_buf).get_unchecked(0)
        };
        let d_type = *buf.get_unchecked(Self::HEADER_SIZE);
        let mut name = [NULL_BYTE; 256];
        let bytes = buf.get_unchecked(Self::NAME_START..);
        for (ind, byte) in bytes.iter().enumerate() {
            let byte = *byte;
            if byte == NULL_BYTE {
                break;
            }
            name[ind] = byte;
        }
        Some(Self {
            d_ino: InoT::from_ne_bytes(
                buf.get_unchecked(0..Self::INOT_SIZE)
                    .try_into()
                    .unwrap_unchecked(),
            ),
            d_off: OffT::from_ne_bytes(
                buf.get_unchecked(Self::INOT_SIZE..Self::INOT_SIZE + Self::OFFT_SIZE)
                    .try_into()
                    .unwrap_unchecked(),
            ),
            d_reclen: len,
            d_name: name,
            d_type,
        })
    }
}

#[cfg(test)]
mod tests {
    use libc::{DT_DIR, DT_REG};

    use crate::linux::get_dents::get_dents;
    use crate::linux::Dirent;
    use crate::string::strlen::buf_strlen;
    use crate::unistd::{open, OpenFlags};

    struct DirentIterator<'a> {
        buf: &'a [u8],
        byte_offset: usize,
        read: usize,
    }

    impl<'a> Iterator for DirentIterator<'a> {
        type Item = Dirent;

        fn next(&mut self) -> Option<Self::Item> {
            if self.read == self.byte_offset {
                return None;
            }
            unsafe {
                Dirent::try_from_bytes(&self.buf[self.byte_offset..]).map(|d| {
                    self.byte_offset += d.d_reclen as usize;
                    d
                })
            }
        }
    }

    #[test]
    fn try_read_dir() {
        let dir = open(
            "test-files/dents-test\0",
            OpenFlags::O_CLOEXEC | OpenFlags::O_RDONLY,
        )
        .unwrap();
        let mut buf = [0u8; 128];
        let read_size = get_dents(dir, &mut buf).unwrap();
        let it = DirentIterator {
            buf: &buf,
            byte_offset: 0,
            read: read_size,
        };
        let find = [
            (24, "d1", DT_DIR),
            (32, "f1.txt", DT_REG),
            (24, ".", DT_DIR),
            (24, "..", DT_DIR),
        ];
        let mut found = [0, 0, 0, 0];
        for (ind, ent) in it.enumerate() {
            // Ordering is random-ish
            for (size, name, dt) in find {
                let found_size = ent.d_reclen;
                let found_name = ent.d_name;
                let name_len = buf_strlen(&found_name).unwrap();
                if size == found_size
                    && name == core::str::from_utf8(&found_name[..name_len]).unwrap()
                    && dt == ent.d_type
                {
                    found[ind] += 1;
                }
            }
        }
        for expect in found {
            assert_eq!(1, expect);
        }
    }
}
