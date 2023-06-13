use crate::platform::numbers::NonNegativeI32;
use core::mem::MaybeUninit;

transparent_bitflags! {
    pub struct SignalKind: NonNegativeI32 {
        const DEFAULT = NonNegativeI32::comptime_checked_new(0);
        const SIGHUP = NonNegativeI32::comptime_checked_new(linux_rust_bindings::signal::SIGHUP);
        const SIGINT = NonNegativeI32::comptime_checked_new(linux_rust_bindings::signal::SIGINT);
        const SIGQUIT = NonNegativeI32::comptime_checked_new(linux_rust_bindings::signal::SIGQUIT);
        const SIGILL = NonNegativeI32::comptime_checked_new(linux_rust_bindings::signal::SIGILL);
        const SIGTRAP = NonNegativeI32::comptime_checked_new(linux_rust_bindings::signal::SIGTRAP);
        const SIGABRT = NonNegativeI32::comptime_checked_new(linux_rust_bindings::signal::SIGABRT);
        const SIGIOT = NonNegativeI32::comptime_checked_new(linux_rust_bindings::signal::SIGIOT);
        const SIGBUS = NonNegativeI32::comptime_checked_new(linux_rust_bindings::signal::SIGBUS);
        const SIGFPE = NonNegativeI32::comptime_checked_new(linux_rust_bindings::signal::SIGFPE);
        const SIGKILL = NonNegativeI32::comptime_checked_new(linux_rust_bindings::signal::SIGKILL);
        const SIGUSR1 = NonNegativeI32::comptime_checked_new(linux_rust_bindings::signal::SIGUSR1);
        const SIGSEGV = NonNegativeI32::comptime_checked_new(linux_rust_bindings::signal::SIGSEGV);
        const SIGUSR2 = NonNegativeI32::comptime_checked_new(linux_rust_bindings::signal::SIGUSR2);
        const SIGPIPE = NonNegativeI32::comptime_checked_new(linux_rust_bindings::signal::SIGPIPE);
        const SIGALRM = NonNegativeI32::comptime_checked_new(linux_rust_bindings::signal::SIGALRM);
        const SIGTERM = NonNegativeI32::comptime_checked_new(linux_rust_bindings::signal::SIGTERM);
        const SIGSTKFLT = NonNegativeI32::comptime_checked_new(linux_rust_bindings::signal::SIGSTKFLT);
        const SIGCHLD = NonNegativeI32::comptime_checked_new(linux_rust_bindings::signal::SIGCHLD);
        const SIGCONT = NonNegativeI32::comptime_checked_new(linux_rust_bindings::signal::SIGCONT);
        const SIGSTOP = NonNegativeI32::comptime_checked_new(linux_rust_bindings::signal::SIGSTOP);
        const SIGTSTP = NonNegativeI32::comptime_checked_new(linux_rust_bindings::signal::SIGTSTP);
        const SIGTTIN = NonNegativeI32::comptime_checked_new(linux_rust_bindings::signal::SIGTTIN);
        const SIGTTOU = NonNegativeI32::comptime_checked_new(linux_rust_bindings::signal::SIGTTOU);
        const SIGURG = NonNegativeI32::comptime_checked_new(linux_rust_bindings::signal::SIGURG);
        const SIGXCPU = NonNegativeI32::comptime_checked_new(linux_rust_bindings::signal::SIGXCPU);
        const SIGXFSZ = NonNegativeI32::comptime_checked_new(linux_rust_bindings::signal::SIGXFSZ);
        const SIGVTALRM = NonNegativeI32::comptime_checked_new(linux_rust_bindings::signal::SIGVTALRM);
        const SIGPROF = NonNegativeI32::comptime_checked_new(linux_rust_bindings::signal::SIGPROF);
        const SIGWINCH = NonNegativeI32::comptime_checked_new(linux_rust_bindings::signal::SIGWINCH);
        const SIGIO = NonNegativeI32::comptime_checked_new(linux_rust_bindings::signal::SIGIO);
        const SIGPOLL = NonNegativeI32::comptime_checked_new(linux_rust_bindings::signal::SIGPOLL);
        const SIGPWR = NonNegativeI32::comptime_checked_new(linux_rust_bindings::signal::SIGPWR);
        const SIGSYS = NonNegativeI32::comptime_checked_new(linux_rust_bindings::signal::SIGSYS);
        const SIGUNUSED = NonNegativeI32::comptime_checked_new(linux_rust_bindings::signal::SIGUNUSED);
        const SIGRTMIN = NonNegativeI32::comptime_checked_new(linux_rust_bindings::signal::SIGRTMIN);
        const SIGSTKSZ = NonNegativeI32::comptime_checked_new(linux_rust_bindings::signal::SIGSTKSZ);
    }
}

transparent_bitflags! {
    pub struct SaMask: i64 {
        const DEFAULT = 0;
        const SA_NOCLDSTOP = linux_rust_bindings::signal::SA_NOCLDSTOP as i64;
        const SA_NOCLDWAIT = linux_rust_bindings::signal::SA_NOCLDWAIT as i64;
        const SA_SIGINFO = linux_rust_bindings::signal::SA_SIGINFO as i64;
        const SA_UNSUPPORTED = linux_rust_bindings::signal::SA_UNSUPPORTED as i64;
        const SA_EXPOSE_TAGBITS = linux_rust_bindings::signal::SA_EXPOSE_TAGBITS as i64;
        const SA_ONSTACK = linux_rust_bindings::signal::SA_ONSTACK as i64;
        const SA_RESTART = linux_rust_bindings::signal::SA_RESTART as i64;
        const SA_NODEFER = linux_rust_bindings::signal::SA_NODEFER as i64;
        const SA_RESETHAND = linux_rust_bindings::signal::SA_RESETHAND;
        const SA_NOMASK = linux_rust_bindings::signal::SA_NOMASK as i64;
        const SA_ONESHOT = linux_rust_bindings::signal::SA_ONESHOT;
        const SA_RESTORER = linux_rust_bindings::signal::SA_RESTORER as i64;
    }
}
/// A set of signals
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SigSetT {
    __val: [MaybeUninit<u64>; 16],
}

impl Default for SigSetT {
    fn default() -> Self {
        Self {
            __val: [
                MaybeUninit::new(0),
                MaybeUninit::uninit(),
                MaybeUninit::uninit(),
                MaybeUninit::uninit(),
                MaybeUninit::uninit(),
                MaybeUninit::uninit(),
                MaybeUninit::uninit(),
                MaybeUninit::uninit(),
                MaybeUninit::uninit(),
                MaybeUninit::uninit(),
                MaybeUninit::uninit(),
                MaybeUninit::uninit(),
                MaybeUninit::uninit(),
                MaybeUninit::uninit(),
                MaybeUninit::uninit(),
                MaybeUninit::uninit(),
            ],
        }
    }
}

pub const SIG_DFL: usize = 0;
pub const SIG_IGN: usize = 1;
