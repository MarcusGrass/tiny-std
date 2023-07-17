use core::str::Utf8Error;

use rusl::string::strlen::strlen;
use rusl::string::unix_str::AsUnixStr;
use rusl::string::unix_str::UnixStr;

/// We have to mimic libc globals here sadly, we rip the environment off the first pointer of the stack
/// in the start method. Should never be modified ever, just set on start
pub(crate) static mut ENV: Env = Env {
    arg_c: 0,
    arg_v: core::ptr::null(),
    env_p: core::ptr::null(),
};

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

pub fn var<P: AsUnixStr>(ident: P) -> Result<&'static str, VarError> {
    let mut env_ptr = unsafe { ENV.env_p };
    while !env_ptr.is_null() {
        unsafe {
            let var_ptr = env_ptr.read();
            // The last ptr in the ptr of ptrs in always null
            if var_ptr.is_null() {
                return Err(VarError::Missing);
            }
            let match_up_to = ident.match_up_to(var_ptr);
            if match_up_to != 0 {
                // Next is '='
                if var_ptr.add(match_up_to + 1).read() == b'=' {
                    let value_len = strlen(var_ptr.add(match_up_to + 2));
                    let value_slice =
                        core::slice::from_raw_parts(var_ptr.add(match_up_to + 2), value_len);
                    return core::str::from_utf8(value_slice).map_err(VarError::NotUnicode);
                }
            }

            env_ptr = env_ptr.add(1);
        }
    }
    Err(VarError::Missing)
}

pub fn args() -> Args {
    Args {
        ind: 0,
        num_args: unsafe { ENV.arg_c } as usize,
    }
}

pub fn args_os() -> ArgsOs {
    ArgsOs {
        ind: 0,
        num_args: unsafe { ENV.arg_c } as usize,
    }
}

pub struct Args {
    ind: usize,
    num_args: usize,
}

impl Iterator for Args {
    type Item = Result<&'static str, Utf8Error>;

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
                let len = strlen(arg);
                let arg_slice = core::slice::from_raw_parts(arg, len);
                return Some(core::str::from_utf8(arg_slice));
            }
        }
        None
    }
}

impl ExactSizeIterator for Args {
    fn len(&self) -> usize {
        self.num_args
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
            self.ind += 1;
            unsafe {
                let arg_ptr = ENV.arg_v.add(self.ind);
                if arg_ptr.is_null() {
                    return None;
                }
                let arg = arg_ptr.read();
                if arg.is_null() {
                    return None;
                }
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
