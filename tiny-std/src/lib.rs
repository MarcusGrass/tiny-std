#![cfg_attr(feature = "start", feature(naked_functions))]
#![cfg_attr(not(test), no_std)]
#[cfg(feature = "alloc")]
extern crate alloc;

pub use rusl::string::unix_str::*;

pub mod env;
pub mod error;
pub mod fs;
pub mod io;
pub mod net;
pub mod process;
pub mod signal;
#[cfg(feature = "start")]
pub mod start;
pub mod thread;
pub mod time;
pub mod unix;
#[cfg(feature = "vdso")]
mod vdso;
