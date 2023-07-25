use linux_rust_bindings::epoll::__poll_t;
/// Some of these consts can't be generated correctly with bindgen, have to do them manually
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
transparent_bitflags! {
    pub struct EpollEventMask: __poll_t {
        const DEFAULT = 0;
        const EPOLLIN	        = 0x0000_0001;
        const EPOLLPRI	        = 0x0000_0002;
        const EPOLLOUT	        = 0x0000_0004;
        const EPOLLERR	        = 0x0000_0008;
        const EPOLLHUP          = 0x0000_0010;
        const EPOLLNVAL 	    = 0x0000_0020;
        const EPOLLRDNORM	    = 0x0000_0040;
        const EPOLLRDBAND	    = 0x0000_0080;
        const EPOLLWRNORM	    = 0x0000_0100;
        const EPOLLWRBAND   	= 0x0000_0200;
        const EPOLLMSG          = 0x0000_0400;
        const EPOLLRDHUP    	= 0x0000_2000;
        const EPOLLEXCLUSIVE    = 1 << 28;
        const EPOLLWAKEUP	    = 1 << 29;
        const EPOLLONESHOT      = 1 << 30;
        const EPOLLET	    	= 1 << 31;
    }
}

#[derive(Debug, Copy, Clone)]
pub enum EpollOp {
    Add,
    Mod,
    Del,
}

impl EpollOp {
    pub(crate) const fn into_op(self) -> i32 {
        match self {
            Self::Add => linux_rust_bindings::epoll::EPOLL_CTL_ADD,
            Self::Mod => linux_rust_bindings::epoll::EPOLL_CTL_MOD,
            Self::Del => linux_rust_bindings::epoll::EPOLL_CTL_DEL,
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct EpollEvent(pub(crate) linux_rust_bindings::epoll::epoll_event);

impl EpollEvent {
    #[inline]
    #[must_use]
    pub const fn new(user_data: u64, mask: EpollEventMask) -> Self {
        Self(linux_rust_bindings::epoll::epoll_event {
            events: mask.bits(),
            data: user_data,
        })
    }

    #[inline]
    #[must_use]
    pub const fn get_data(&self) -> u64 {
        self.0.data
    }

    #[inline]
    #[must_use]
    pub const fn get_events(&self) -> EpollEventMask {
        EpollEventMask(self.0.events)
    }
}
