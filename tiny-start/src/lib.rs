//! A crate for code that cannot be disturbed by builtins for one of two reasons:
//! 1. Builtins are provided here, and the compiler now causes infinite recursion
//! when providing your own symbols without `![no_builtins]` [see issue](https://github.com/rust-lang/rust/issues/115225)
//! 2. Code that runs before symbol resolution lives here, which would otherwise provoke memset insertions
//! in some cases, which would result in a segfault if symbols are relocated
//!
#![no_std]
#![no_builtins]
#![warn(clippy::pedantic)]
#![allow(
    clippy::inline_always,
    clippy::module_name_repetitions,
    clippy::similar_names
)]

#[cfg(feature = "aux")]
pub mod elf;

#[cfg(feature = "start")]
pub mod start;

#[cfg(feature = "mem-symbols")]
pub mod symbols;
