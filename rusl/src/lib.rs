#![allow(unused_doc_comments)]
#![cfg_attr(not(test), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub use error::{Error, Result};

pub mod error;
#[macro_use]
pub(crate) mod macros;
pub mod network;
pub mod platform;
pub mod process;
pub mod select;
pub mod string;
pub mod time;
pub mod unistd;
