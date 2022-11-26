pub use clone::{clone3, CloneArgs, CloneFlags};
pub use execve::execve;
pub use exit::exit;
pub use fork::fork;
pub use wait::wait_pid;

mod execve;
mod exit;

mod clone;
mod fork;
mod wait;
