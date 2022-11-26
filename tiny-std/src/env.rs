#[cfg(feature = "start")]
use core::str::Utf8Error;

#[cfg(feature = "start")]
use rusl::compat::unix_str::AsUnixStr;
#[cfg(feature = "start")]
use rusl::compat::unix_str::UnixStr;
#[cfg(any(feature = "start", feature = "alloc"))]
use rusl::string::strlen::strlen;

#[cfg(feature = "start")]
use crate::start::ENV;

#[cfg(feature = "start")]
#[derive(Debug, Copy, Clone)]
pub enum VarError {
    Missing,
    NotUnicode(Utf8Error),
}

/// Attempts to get the system hostname as a utf8 `String`
/// # Errors
/// Hostname is not utf8
#[cfg(feature = "alloc")]
pub fn host_name() -> Result<alloc::string::String, crate::error::Error> {
    let raw = rusl::unistd::uname()?.nodename;
    let strlen = unsafe { strlen(raw.as_ptr()) };

    alloc::string::String::from_utf8(raw[..strlen].to_vec())
        .map_err(|_| crate::error::Error::no_code("Failed to convert hostname to string"))
}

#[cfg(feature = "start")]
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

#[cfg(feature = "start")]
pub fn args() -> Args {
    Args {
        ind: 0,
        num_args: unsafe { ENV.arg_c } as usize,
    }
}

#[cfg(feature = "start")]
pub fn args_os() -> ArgsOs {
    ArgsOs {
        ind: 0,
        num_args: unsafe { ENV.arg_c } as usize,
    }
}

#[cfg(feature = "start")]
pub struct Args {
    ind: usize,
    num_args: usize,
}

#[cfg(feature = "start")]
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

#[cfg(feature = "start")]
impl ExactSizeIterator for Args {
    fn len(&self) -> usize {
        self.num_args
    }
}

#[cfg(feature = "start")]
pub struct ArgsOs {
    ind: usize,
    num_args: usize,
}

#[cfg(feature = "start")]
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

#[cfg(feature = "start")]
impl ExactSizeIterator for ArgsOs {
    fn len(&self) -> usize {
        self.num_args
    }
}
