use sc::syscall;

use crate::platform::Fd;

/// Reads directory entities into the provided buffer and returns the number of bytes read
/// See [Linux docs for details](https://man7.org/linux/man-pages/man2/getdents.2.html)
/// # Errors
/// See above
pub fn get_dents(fd: Fd, dir_p: &mut [u8]) -> crate::Result<usize> {
    let res = unsafe { syscall!(GETDENTS64, fd, dir_p.as_mut_ptr(), dir_p.len()) };
    bail_on_below_zero!(res, "`GETDENTS64` syscall failed");
    Ok(res)
}

#[cfg(test)]
mod tests {
    use crate::platform::{DirType, Dirent, OpenFlags};
    use crate::string::strlen::buf_strlen;
    use crate::unistd::open;

    use super::*;

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
            (24, "d1", DirType::DT_DIR),
            (32, "f1.txt", DirType::DT_REG),
            (24, ".", DirType::DT_DIR),
            (24, "..", DirType::DT_DIR),
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
