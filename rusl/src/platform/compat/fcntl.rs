#[derive(Debug, Copy, Clone)]
pub enum FcntlDup {
    Dup,
    DupCloexec
}

impl FcntlDup {
    pub(crate) const fn into_cmd(self) -> i32 {
        match self {
            Self::Dup => linux_rust_bindings::fcntl::F_DUPFD,
            Self::DupCloexec => linux_rust_bindings::fcntl::F_DUPFD_CLOEXEC,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum FcntlFileStatusCmd {
    Get,
    Set,
}

impl FcntlFileStatusCmd {
    pub(crate) const fn into_cmd(self) -> i32 {
        match self {
            Self::Get => linux_rust_bindings::fcntl::F_GETFL,
            Self::Set => linux_rust_bindings::fcntl::F_SETFL,
        }
    }
}