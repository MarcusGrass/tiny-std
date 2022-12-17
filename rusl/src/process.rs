pub use clone::{clone, clone3, fork};
pub use execve::execve;
pub use exit::exit;
pub use wait::wait_pid;

mod execve;
mod exit;

mod clone;
#[cfg(test)]
mod test;
mod wait;
