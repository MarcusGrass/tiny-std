use rusl::platform::{Fd, GidT, OpenFlags, UidT};

use crate::error::{Error, Result};

#[derive(Debug, Copy, Clone)]
pub struct Passwd<'a> {
    pub name: &'a str,
    pub passwd: &'a str,
    pub uid: UidT,
    pub gid: GidT,
    pub gecos: &'a str,
    pub dir: &'a str,
    pub shell: &'a str,
}

/// Attempts to get a `Passwd` entry by pwuid
/// # Errors
/// `uid` isn't listed in `/etc/passwd`
/// `/etc/passwd` isn't readable.
pub fn getpwuid_r(uid: UidT, buf: &mut [u8]) -> Result<Option<Passwd>> {
    let fd =
        unsafe { rusl::unistd::open_raw(c"/etc/passwd".as_ptr() as usize, OpenFlags::O_RDONLY)? };
    search_pwd_fd(fd, uid, buf)
}

#[inline]
fn search_pwd_fd(fd: Fd, uid: UidT, buf: &mut [u8]) -> Result<Option<Passwd>> {
    // Compiler gets confused here, or I am causing UB, one of the two.
    // --- Mut borrow start
    rusl::unistd::read(fd, buf)?;
    // --- Borrow ends
    loop {
        // --- Immut borrow start
        let buf_view = unsafe { core::slice::from_raw_parts(buf.as_ptr(), buf.len()) };
        // When this returns we've dropped the immutable borrow and can overwrite the
        // bytes that we're discarding.
        let b = match search_from(uid, buf_view)? {
            // Returning bytes borrowed from `buf` but the compiler things this entry still claims
            // those bytes and doesn't allow us to mutate the buffer again
            SearchRes::Pwd(p) => return Ok(Some(p)),
            SearchRes::ReadUpTo(b) => b,
            SearchRes::NotFound => return Ok(None),
        };
        // --- Immut borrow ends
        let len = buf.len();
        buf.copy_within(b.., 0);
        // --- Mut borrow start
        rusl::unistd::read(fd, &mut buf[len - b..])?;
        // --- Mut borrow ends
    }
}

enum SearchRes<'a> {
    Pwd(Passwd<'a>),
    ReadUpTo(usize),
    NotFound,
}

#[inline]
fn search_from(uid: UidT, buf: &[u8]) -> Result<SearchRes> {
    if let Some(pwd) = find_by_uid(buf, uid)? {
        Ok(SearchRes::Pwd(pwd))
    } else if let Some(nl) = find_last_newline(buf) {
        Ok(SearchRes::ReadUpTo(nl + 1))
    } else {
        Ok(SearchRes::NotFound)
    }
}

#[inline]
fn find_by_uid(pwd_buf: &[u8], uid: UidT) -> Result<Option<Passwd>> {
    let mut offset = 0;
    loop {
        let Some(next) = next_line(&pwd_buf[offset..]) else {
            return Ok(None);
        };
        let res = try_pwd(next)?;
        if let Some(pwd) = res {
            if pwd.uid == uid {
                return Ok(Some(pwd));
            }
        }
        offset += next.len() + 1;
    }
}

#[inline]
fn next_line(buf: &[u8]) -> Option<&[u8]> {
    for i in 0..buf.len() {
        if buf[i] == b'\n' {
            return Some(&buf[..i]);
        }
    }
    None
}

#[inline]
fn try_pwd(line: &[u8]) -> Result<Option<Passwd>> {
    let mut slices = [0, 0, 0, 0, 0, 0];
    for (ind, byte) in line.iter().enumerate() {
        if *byte == b':' {
            for i in &mut slices {
                if *i == 0 {
                    *i = ind;
                    break;
                }
            }
        }
    }
    if slices[5] == 0 {
        return Ok(None);
    }
    let mut shell_line = core::str::from_utf8(&line[slices[5] + 1..])
        .map_err(|_| Error::no_code("Failed to convert pwd shell to utf8"))?;
    if shell_line.ends_with('\n') {
        shell_line = shell_line.trim_end_matches('\n');
    }
    Ok(Some(Passwd {
        name: core::str::from_utf8(&line[..slices[0]])
            .map_err(|_| Error::no_code("Failed to convert pwd name to utf8"))?,
        passwd: core::str::from_utf8(&line[slices[0] + 1..slices[1]])
            .map_err(|_| Error::no_code("Failed to convert pwd passwd to utf8"))?,
        uid: try_parse_num(&line[slices[1] + 1..slices[2]])?,
        gid: try_parse_num(&line[slices[2] + 1..slices[3]])?,
        gecos: core::str::from_utf8(&line[slices[3] + 1..slices[4]])
            .map_err(|_| Error::no_code("Failed to convert pwd gecos to utf8"))?,
        dir: core::str::from_utf8(&line[slices[4] + 1..slices[5]])
            .map_err(|_| Error::no_code("Failed to convert pwd dir to utf8"))?,
        shell: shell_line,
    }))
}

const NUM_OUT_OF_RANGE: Error = Error::no_code("Number out of range");

// Just ascii numbers
#[inline]
#[expect(clippy::needless_range_loop)]
fn try_parse_num(buf: &[u8]) -> Result<u32> {
    let len = buf.len();
    let mut pow = u32::try_from(buf.len()).map_err(|_e| {
        Error::no_code("Tried to parse a number that had a digit-length larger than u32::MAX")
    })?;
    let mut sum: u32 = 0;
    for i in 0..len {
        pow -= 1;
        let digit = buf[i].checked_sub(48).ok_or(Error::no_code(
            "Unexpected value in buffer to parse as number.",
        ))?;
        sum = sum
            .checked_add(
                u32::from(digit)
                    .checked_mul(10u32.checked_pow(pow).ok_or(NUM_OUT_OF_RANGE)?)
                    .ok_or(NUM_OUT_OF_RANGE)?,
            )
            .ok_or(NUM_OUT_OF_RANGE)?;
    }
    Ok(sum)
}

#[inline]
fn find_last_newline(buf: &[u8]) -> Option<usize> {
    let len = buf.len();
    for i in 1..len {
        if buf[len - i] == b'\n' {
            return Some(len - i);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use rusl::platform::OpenFlags;
    use rusl::string::unix_str::UnixStr;
    use rusl::unistd::open;

    use crate::unix::passwd::getpw_r::{
        find_by_uid, next_line, search_pwd_fd, try_parse_num, try_pwd,
    };

    const EXAMPLE: &str = "root:x:0:0::/root:/bin/bash
bin:x:1:1::/:/usr/bin/nologin
daemon:x:2:2::/:/usr/bin/nologin
mail:x:8:12::/var/spool/mail:/usr/bin/nologin
ftp:x:14:11::/srv/ftp:/usr/bin/nologin
http:x:33:33::/srv/http:/usr/bin/nologin
nobody:x:65534:65534:Nobody:/:/usr/bin/nologin
dbus:x:81:81:System Message Bus:/:/usr/bin/nologin
systemd-journal-remote:x:981:981:systemd Journal Remote:/:/usr/bin/nologin
systemd-network:x:980:980:systemd Network Management:/:/usr/bin/nologin
systemd-oom:x:979:979:systemd Userspace OOM Killer:/:/usr/bin/nologin
systemd-resolve:x:978:978:systemd Resolver:/:/usr/bin/nologin
systemd-timesync:x:977:977:systemd Time Synchronization:/:/usr/bin/nologin
systemd-coredump:x:976:976:systemd Core Dumper:/:/usr/bin/nologin
uuidd:x:68:68::/:/usr/bin/nologin
git:x:975:975:git daemon user:/:/usr/bin/git-shell
dhcpcd:x:974:974:dhcpcd privilege separation:/:/usr/bin/nologin
gramar:x:1000:1000::/home/gramar:/usr/bin/zsh
nvidia-persis";

    #[test]
    fn line_by_line() {
        let buf = EXAMPLE.as_bytes();
        let mut offset = 0;
        for line in EXAMPLE.lines() {
            if let Some(next) = next_line(&buf[offset..]) {
                assert_eq!(
                    line.as_bytes(),
                    next,
                    "\n{line} vs {}",
                    core::str::from_utf8(next).unwrap()
                );
                offset += line.len() + 1; // newline + 1
            } else {
                assert_eq!("nvidia-persis", line);
            }
        }
    }

    #[test]
    fn pwd_line() {
        let line = b"bin:x:1:1::/:/usr/bin/nologin\n";
        let pwd = try_pwd(line).unwrap().unwrap();
        assert_eq!("bin", pwd.name);
        assert_eq!("x", pwd.passwd);
        assert_eq!(1, pwd.uid);
        assert_eq!(1, pwd.gid);
        assert_eq!("", pwd.gecos);
        assert_eq!("/", pwd.dir);
        assert_eq!("/usr/bin/nologin", pwd.shell);
    }

    #[test]
    fn parse_num() {
        let my_num = "2048";
        assert_eq!(2048, try_parse_num(my_num.as_bytes()).unwrap());
        let my_num = "0";
        assert_eq!(0, try_parse_num(my_num.as_bytes()).unwrap());
        let my_num = "-5";
        assert!(try_parse_num(my_num.as_bytes()).is_err());
        // u64::Max
        assert!(try_parse_num("18446744073709551615".as_bytes()).is_err());
    }

    #[test]
    fn pwd_by_uid() {
        let pwd = find_by_uid(EXAMPLE.as_bytes(), 1000).unwrap().unwrap();
        assert_eq!(pwd.name, "gramar");
        assert_eq!(pwd.passwd, "x");
        assert_eq!(pwd.uid, 1000);
        assert_eq!(pwd.gid, 1000);
        assert_eq!(pwd.gecos, "");
        assert_eq!(pwd.dir, "/home/gramar");
        assert_eq!(pwd.shell, "/usr/bin/zsh");
    }

    #[test]
    fn pwd_by_missing() {
        assert!(find_by_uid(EXAMPLE.as_bytes(), 9959).unwrap().is_none());
    }

    #[test]
    fn search_pwd_normal_sized_buf() {
        let fd = open(
            UnixStr::try_from_str("test-files/unix/passwd/pwd_test.txt\0").unwrap(),
            OpenFlags::O_RDONLY,
        )
        .unwrap();
        let mut buf = [0u8; 1024];
        let pwd = search_pwd_fd(fd, 1000, &mut buf).unwrap().unwrap();
        assert_eq!(pwd.name, "gramar");
        assert_eq!(pwd.passwd, "x");
        assert_eq!(pwd.uid, 1000);
        assert_eq!(pwd.gid, 1000);
        assert_eq!(pwd.gecos, "");
        assert_eq!(pwd.dir, "/home/gramar");
        assert_eq!(pwd.shell, "/usr/bin/zsh");
    }

    #[test]
    fn search_pwd_small_buf() {
        let fd = open(
            UnixStr::try_from_str("test-files/unix/passwd/pwd_test.txt\0").unwrap(),
            OpenFlags::O_RDONLY,
        )
        .unwrap();
        let mut buf = [0u8; 256];
        let pwd = search_pwd_fd(fd, 1000, &mut buf).unwrap().unwrap();
        assert_eq!(pwd.name, "gramar");
        assert_eq!(pwd.passwd, "x");
        assert_eq!(pwd.uid, 1000);
        assert_eq!(pwd.gid, 1000);
        assert_eq!(pwd.gecos, "");
        assert_eq!(pwd.dir, "/home/gramar");
        assert_eq!(pwd.shell, "/usr/bin/zsh");
    }

    #[test]
    fn last_entry_tiny_buf() {
        let fd = open(
            UnixStr::try_from_str("test-files/unix/passwd/pwd_test.txt\0").unwrap(),
            OpenFlags::O_RDONLY,
        )
        .unwrap();
        let mut buf = [0u8; 100];
        let pwd = search_pwd_fd(fd, 110, &mut buf).unwrap().unwrap();
        assert_eq!(pwd.name, "partimag");
        assert_eq!(pwd.passwd, "x");
        assert_eq!(pwd.uid, 110);
        assert_eq!(pwd.gid, 110);
        assert_eq!(pwd.gecos, "Partimage user");
        assert_eq!(pwd.dir, "/");
        assert_eq!(pwd.shell, "/usr/bin/nologin");
    }
}
