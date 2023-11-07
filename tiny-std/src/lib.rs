#![cfg_attr(not(test), no_std)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::let_underscore_untyped,
    clippy::module_name_repetitions,
    clippy::similar_names,
    clippy::inline_always
)]
#[cfg(feature = "alloc")]
extern crate alloc;

pub use error::{Error, Result};
pub use rusl::error::Errno;
pub use rusl::string::unix_str::*;
pub use rusl::unix_lit;
pub use rusl::Error as RuslError;

#[cfg(feature = "allocator-provided")]
pub mod allocator;
pub mod elf;
#[cfg(feature = "start")]
pub mod env;
mod error;
pub mod fs;
pub mod io;
pub mod linux;
pub mod net;
pub mod process;
#[cfg(feature = "start")]
pub mod start;
pub mod sync;
pub mod thread;
pub mod time;
pub mod unix;
