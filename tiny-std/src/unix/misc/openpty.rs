use rusl::ioctl::ioctl;
use rusl::platform::{Fd, OpenFlags, SetAction, TermioFlags, Termios, WindowSize};
use rusl::string::unix_str::UnixStr;
use rusl::termios::tcsetattr;
use rusl::unistd::{open, open_raw};

#[derive(Debug, Copy, Clone)]
pub struct TerminalHandle {
    pub master: Fd,
    pub slave: Fd,
}

/// Attempts to open a pty returning the handles to it
/// # Errors
/// Not many errors can occur assuming that (`None`, `None`, `None`) is passed and you have
/// appropriate permissions.
/// See the [linux docs for the exceptions](https://man7.org/linux/man-pages/man2/ioctl_tty.2.html)
pub fn openpty(
    name: Option<&UnixStr>,
    termios: Option<&Termios>,
    winsize: Option<&WindowSize>,
) -> crate::error::Result<TerminalHandle> {
    const PTMX: &UnixStr = UnixStr::from_str_checked("/dev/ptmx\0");
    let use_flags: OpenFlags = OpenFlags::O_RDWR | OpenFlags::O_NOCTTY;
    unsafe {
        let master = open(PTMX, use_flags)?;
        let mut pty_num = 0;
        let pty_num_addr = core::ptr::addr_of_mut!(pty_num);
        // Todo: Maybe check if not zero and bail like musl does
        ioctl(
            master,
            TermioFlags::TIOCSPTLCK.bits(),
            pty_num_addr as usize,
        )?;
        ioctl(master, TermioFlags::TIOCGPTN.bits(), pty_num_addr as usize)?;
        let slave = if let Some(name) = name {
            open(name, use_flags)?
        } else {
            let bytename: u8 = pty_num.try_into().map_err(|_| {
                crate::error::Error::no_code("Terminal number exceeded u8::MAX or was negative")
            })?;
            // To do this without an allocator have to format this string manually
            // on the stack.
            let name = create_pty_name(bytename);
            open_raw(core::ptr::addr_of!(name) as usize, use_flags)?
        };
        if let Some(tio) = termios {
            tcsetattr(slave, SetAction::NOW, tio)?;
        }
        if let Some(winsize) = winsize {
            ioctl(
                slave,
                TermioFlags::TIOCSWINSZ.bits(),
                core::ptr::addr_of!(winsize) as usize,
            )?;
        }
        Ok(TerminalHandle { master, slave })
    }
}

#[derive(Debug, Copy, Clone)]
enum ByteChars {
    One(u8),
    Two([u8; 2]),
    Three([u8; 3]),
}

#[inline]
fn create_pty_name(pty_num: u8) -> [u8; 13] {
    let mut name = *b"/dev/pts/0\0\0\0";
    match get_chars(pty_num) {
        ByteChars::One(byte) => {
            name[9] = byte;
            name
        }
        ByteChars::Two([b1, b2]) => {
            name[9] = b1;
            name[10] = b2;
            name
        }
        ByteChars::Three([b1, b2, b3]) => {
            name[9] = b1;
            name[10] = b2;
            name[11] = b3;
            name
        }
    }
}

fn get_chars(num: u8) -> ByteChars {
    if num < 10 {
        ByteChars::One(num + 48)
    } else if num < 100 {
        let rem = num % 10;
        let base = num / 10;
        ByteChars::Two([base + 48, rem + 48])
    } else {
        let base = num / 100;
        let next_base = num - base * 100;
        let nb = next_base / 10;
        let rem = next_base % 10;
        ByteChars::Three([base + 48, nb + 48, rem + 48])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrite_name() {
        for i in 0..u8::MAX {
            check_name(i);
        }
    }

    fn check_name(num: u8) {
        let n = create_pty_name(num);
        if num < 10 {
            assert_eq!(std::format!("/dev/pts/{num}\0\0\0").as_bytes(), &n);
        } else if num < 100 {
            assert_eq!(std::format!("/dev/pts/{num}\0\0").as_bytes(), &n);
        } else {
            assert_eq!(std::format!("/dev/pts/{num}\0").as_bytes(), &n);
        }
    }

    #[test]
    fn rewrite_single() {
        let bc = get_chars(8);
        if let ByteChars::One(b) = bc {
            assert_eq!('8', b as char);
        } else {
            panic!("Bad match");
        }
    }

    #[test]
    fn rewrite_double() {
        let bc = get_chars(59);
        if let ByteChars::Two([c1, c2]) = bc {
            assert_eq!('5', c1 as char);
            assert_eq!('9', c2 as char);
        } else {
            panic!("Bad match");
        }
    }

    #[test]
    fn rewrite_triple() {
        let bc = get_chars(231);
        if let ByteChars::Three([c1, c2, c3]) = bc {
            assert_eq!('2', c1 as char);
            assert_eq!('3', c2 as char);
            assert_eq!('1', c3 as char);
        } else {
            panic!("Bad match");
        }
    }
}
