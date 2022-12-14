pub use clone::{clone, CloneArgs, clone3, Clone3Args, CloneFlags, fork};
pub use execve::execve;
pub use exit::exit;
pub use wait::wait_pid;

mod execve;
mod exit;

mod clone;
mod wait;
#[cfg(test)]
mod test;
