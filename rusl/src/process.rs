pub use clone::{clone, clone3, fork};
pub use execve::execve;
pub use exit::exit;
pub use signal::{add_signal_action, CatchSignal, SaSignalaction, SigInfo};
pub use wait::wait_pid;

mod execve;
mod exit;

mod clone;
mod signal;
#[cfg(test)]
mod test;
mod wait;
