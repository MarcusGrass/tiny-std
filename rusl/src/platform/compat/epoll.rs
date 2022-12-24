use linux_rust_bindings::epoll::__poll_t;
/// Some of these consts can't be generated correctly with bindgen, have to do them manually
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
transparent_bitflags! {
    pub struct EpollEventMask: __poll_t {
        const EPOLLIN	        = 0x00000001;
        const EPOLLPRI	        = 0x00000002;
        const EPOLLOUT	        = 0x00000004;
        const EPOLLERR	        = 0x00000008;
        const EPOLLHUP          = 0x00000010;
        const EPOLLNVAL 	    = 0x00000020;
        const EPOLLRDNORM	    = 0x00000040;
        const EPOLLRDBAND	    = 0x00000080;
        const EPOLLWRNORM	    = 0x00000100;
        const EPOLLWRBAND   	= 0x00000200;
        const EPOLLMSG          = 0x00000400;
        const EPOLLRDHUP    	= 0x00002000;
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
    Del
}

impl EpollOp {
    pub(crate) fn into_op(self) -> i32 {
        match self {
            Self::Add => linux_rust_bindings::epoll::EPOLL_CTL_ADD,
            Self::Mod => linux_rust_bindings::epoll::EPOLL_CTL_MOD,
            Self::Del => linux_rust_bindings::epoll::EPOLL_CTL_DEL,
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct EpollEvent(pub(crate)linux_rust_bindings::epoll::epoll_event);

impl EpollEvent {
    #[inline]
    #[must_use]
    pub fn new(user_data: u64, mask: EpollEventMask) -> Self {
        Self(linux_rust_bindings::epoll::epoll_event {
            events: mask.bits(),
            data: user_data,
        })
    }

    #[inline]
    #[must_use]
    pub fn get_data(&self) -> u64 {
        self.0.data
    }

    #[inline]
    #[must_use]
    pub fn get_events(&self) -> EpollEventMask {
        EpollEventMask(self.0.events)
    }
}
