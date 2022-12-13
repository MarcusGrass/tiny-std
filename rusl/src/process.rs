pub use clone::{clone, CloneArgs, clone3, Clone3Args, CloneFlags};
pub use execve::execve;
pub use exit::exit;
pub use fork::fork;
pub use wait::wait_pid;

mod execve;
mod exit;

mod clone;
mod fork;
mod wait;
#[cfg(test)]
mod test;
