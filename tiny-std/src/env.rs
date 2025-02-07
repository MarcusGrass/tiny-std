use core::str::Utf8Error;

use crate::error::Error;
use rusl::string::strlen::strlen;
use rusl::string::unix_str::UnixStr;

/// We have to mimic libc globals here sadly, we rip the environment off the first pointer of the stack
/// in the start method. Should never be modified ever, just set on start
pub(crate) static mut ENV: Env = Env {
    arg_c: 0,
    arg_v: core::ptr::null(),
    env_p: core::ptr::null(),
};

#[expect(clippy::struct_field_names)]
pub(crate) struct Env {
    pub(crate) arg_c: u64,
    pub(crate) arg_v: *const *const u8,
    pub(crate) env_p: *const *const u8,
}

#[derive(Debug, Copy, Clone)]
pub enum VarError {
    Missing,
    NotUnicode(Utf8Error),
}

/// Get a variable for this process' environment with the provided `key`.
/// # Errors
/// 1. Value is not in the environment
/// 2. Value exists but is not utf-8
pub fn var_unix(key: &UnixStr) -> Result<&'static UnixStr, VarError> {
    let mut env_ptr = unsafe { ENV.env_p };
    while !env_ptr.is_null() {
        unsafe {
            let var_ptr = env_ptr.read();
            // The last ptr in the ptr of ptrs in always null
            if var_ptr.is_null() {
                return Err(VarError::Missing);
            }
            let match_up_to = key.match_up_to(UnixStr::from_ptr(var_ptr));
            if match_up_to != 0 {
                // Next is '='
                if var_ptr.add(match_up_to).read() == b'=' {
                    // # Safety
                    // Trusting the OS to null terminate
                    return Ok(UnixStr::from_ptr(var_ptr.add(match_up_to + 1)));
                }
            }

            env_ptr = env_ptr.add(1);
        }
    }
    Err(VarError::Missing)
}

/// Get a variable for this process' environment with the provided `key`.
/// # Errors
/// 1. Value is not in the environment
/// 2. Value exists but is not utf-8
pub fn var(key: &str) -> Result<&'static str, VarError> {
    let mut env_ptr = unsafe { ENV.env_p };
    while !env_ptr.is_null() {
        unsafe {
            let var_ptr = env_ptr.read();
            // The last ptr in the ptr of ptrs in always null
            if var_ptr.is_null() {
                return Err(VarError::Missing);
            }
            let match_up_to = UnixStr::from_ptr(var_ptr).match_up_to_str(key);
            if match_up_to != 0 {
                // Next is '='
                if var_ptr.add(match_up_to).read() == b'=' {
                    let value_len = strlen(var_ptr.add(match_up_to + 1));
                    let value_slice =
                        core::slice::from_raw_parts(var_ptr.add(match_up_to + 1), value_len);
                    return core::str::from_utf8(value_slice).map_err(VarError::NotUnicode);
                }
            }

            env_ptr = env_ptr.add(1);
        }
    }
    Err(VarError::Missing)
}

#[inline]
#[must_use]
pub fn args() -> Args {
    Args(args_os())
}

#[inline]
#[must_use]
#[expect(clippy::cast_possible_truncation)]
pub fn args_os() -> ArgsOs {
    ArgsOs {
        ind: 0,
        num_args: unsafe { ENV.arg_c } as usize,
    }
}

pub struct Args(ArgsOs);

impl Iterator for Args {
    type Item = Result<&'static str, Error>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|e| Ok(UnixStr::as_str(e)?))
    }
}

impl ExactSizeIterator for Args {
    fn len(&self) -> usize {
        self.0.num_args
    }
}

pub struct ArgsOs {
    ind: usize,
    num_args: usize,
}

impl Iterator for ArgsOs {
    type Item = &'static UnixStr;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ind < self.num_args {
            unsafe {
                let arg_ptr = ENV.arg_v.add(self.ind);
                self.ind += 1;
                if arg_ptr.is_null() {
                    return None;
                }
                let arg = arg_ptr.read();
                if arg.is_null() {
                    return None;
                }
                // # Safety:
                // Trusting the OS to null terminate strings
                return Some(UnixStr::from_ptr(arg));
            }
        }
        None
    }
}

impl ExactSizeIterator for ArgsOs {
    fn len(&self) -> usize {
        self.num_args
    }
}
