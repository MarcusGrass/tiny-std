use core::mem::MaybeUninit;

transparent_bitflags! {
    pub struct SignalKind: i32 {
        const SIGHUP = linux_rust_bindings::signal::SIGHUP;
        const SIGINT = linux_rust_bindings::signal::SIGINT;
        const SIGQUIT = linux_rust_bindings::signal::SIGQUIT;
        const SIGILL = linux_rust_bindings::signal::SIGILL;
        const SIGTRAP = linux_rust_bindings::signal::SIGTRAP;
        const SIGABRT = linux_rust_bindings::signal::SIGABRT;
        const SIGIOT = linux_rust_bindings::signal::SIGIOT;
        const SIGBUS = linux_rust_bindings::signal::SIGBUS;
        const SIGFPE = linux_rust_bindings::signal::SIGFPE;
        const SIGKILL = linux_rust_bindings::signal::SIGKILL;
        const SIGUSR1 = linux_rust_bindings::signal::SIGUSR1;
        const SIGSEGV = linux_rust_bindings::signal::SIGSEGV;
        const SIGUSR2 = linux_rust_bindings::signal::SIGUSR2;
        const SIGPIPE = linux_rust_bindings::signal::SIGPIPE;
        const SIGALRM = linux_rust_bindings::signal::SIGALRM;
        const SIGTERM = linux_rust_bindings::signal::SIGTERM;
        const SIGSTKFLT = linux_rust_bindings::signal::SIGSTKFLT;
        const SIGCHLD = linux_rust_bindings::signal::SIGCHLD;
        const SIGCONT = linux_rust_bindings::signal::SIGCONT;
        const SIGSTOP = linux_rust_bindings::signal::SIGSTOP;
        const SIGTSTP = linux_rust_bindings::signal::SIGTSTP;
        const SIGTTIN = linux_rust_bindings::signal::SIGTTIN;
        const SIGTTOU = linux_rust_bindings::signal::SIGTTOU;
        const SIGURG = linux_rust_bindings::signal::SIGURG;
        const SIGXCPU = linux_rust_bindings::signal::SIGXCPU;
        const SIGXFSZ = linux_rust_bindings::signal::SIGXFSZ;
        const SIGVTALRM = linux_rust_bindings::signal::SIGVTALRM;
        const SIGPROF = linux_rust_bindings::signal::SIGPROF;
        const SIGWINCH = linux_rust_bindings::signal::SIGWINCH;
        const SIGIO = linux_rust_bindings::signal::SIGIO;
        const SIGPOLL = linux_rust_bindings::signal::SIGPOLL;
        const SIGPWR = linux_rust_bindings::signal::SIGPWR;
        const SIGSYS = linux_rust_bindings::signal::SIGSYS;
        const SIGUNUSED = linux_rust_bindings::signal::SIGUNUSED;
        const SIGRTMIN = linux_rust_bindings::signal::SIGRTMIN;
        const SIGSTKSZ = linux_rust_bindings::signal::SIGSTKSZ;
    }
}

transparent_bitflags! {
    pub struct SaMask: i64 {
        const SA_NOCLDSTOP = linux_rust_bindings::signal::SA_NOCLDSTOP as i64;
        const SA_NOCLDWAIT = linux_rust_bindings::signal::SA_NOCLDWAIT as i64;
        const SA_SIGINFO = linux_rust_bindings::signal::SA_SIGINFO as i64;
        const SA_UNSUPPORTED = linux_rust_bindings::signal::SA_UNSUPPORTED as i64;
        const SA_EXPOSE_TAGBITS = linux_rust_bindings::signal::SA_EXPOSE_TAGBITS as i64;
        const SA_ONSTACK = linux_rust_bindings::signal::SA_ONSTACK as i64;
        const SA_RESTART = linux_rust_bindings::signal::SA_RESTART as i64;
        const SA_NODEFER = linux_rust_bindings::signal::SA_NODEFER as i64;
        const SA_RESETHAND = linux_rust_bindings::signal::SA_RESETHAND as i64;
        const SA_NOMASK = linux_rust_bindings::signal::SA_NOMASK as i64;
        const SA_ONESHOT = linux_rust_bindings::signal::SA_ONESHOT as i64;
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
