transparent_bitflags! {
    pub struct WaitFlags: i32 {
        const DEFAULT = 0;
        const WNOHANG = linux_rust_bindings::wait::WNOHANG;
        const WUNTRACED = linux_rust_bindings::wait::WUNTRACED;
        const WSTOPPED = linux_rust_bindings::wait::WSTOPPED;
        const WEXITED = linux_rust_bindings::wait::WEXITED;
        const WCONTINUED = linux_rust_bindings::wait::WCONTINUED;
        const WNOWAIT = linux_rust_bindings::wait::WNOWAIT;
    }
}
