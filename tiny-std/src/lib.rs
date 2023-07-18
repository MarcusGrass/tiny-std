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

pub use rusl::string::unix_str::*;

pub mod elf;
#[cfg(feature = "start")]
pub mod env;
pub mod error;
pub mod fs;
pub mod io;
pub mod linux;
pub mod net;
pub mod process;
pub mod rwlock;
#[cfg(feature = "start")]
pub mod start;
pub mod thread;
pub mod time;
pub mod unix;
