// Got this from `musl` gotta see if I can find it in the kernel source
pub const AUX_CNT: usize = 42;

transparent_bitflags! {
    pub struct AuxValue: usize {
        const AT_NULL =   linux_rust_bindings::aux::AT_NULL as usize;	/* end of vector */
        const AT_IGNORE = linux_rust_bindings::aux::AT_IGNORE as usize;	/* entry should be ignored */
        const AT_EXECFD = linux_rust_bindings::aux::AT_EXECFD as usize;	/* file descriptor of program */
        const AT_PHDR =   linux_rust_bindings::aux::AT_PHDR as usize;	/* program headers for program */
        const AT_PHENT =  linux_rust_bindings::aux::AT_PHENT as usize;	/* size of program header entry */
        const AT_PHNUM =  linux_rust_bindings::aux::AT_PHNUM as usize;	/* number of program headers */
        const AT_PAGESZ = linux_rust_bindings::aux::AT_PAGESZ as usize;	/* system page size */
        const AT_BASE =   linux_rust_bindings::aux::AT_BASE as usize;	/* base address of interpreter */
        const AT_FLAGS =  linux_rust_bindings::aux::AT_FLAGS as usize;	/* flags */
        const AT_ENTRY =  linux_rust_bindings::aux::AT_ENTRY as usize;	/* entry point of program */
        const AT_NOTELF = linux_rust_bindings::aux::AT_NOTELF as usize;	/* program is not ELF */
        const AT_UID =    linux_rust_bindings::aux::AT_UID as usize;	    /* real uid */
        const AT_EUID =   linux_rust_bindings::aux::AT_EUID as usize;	/* effective uid */
        const AT_GID =    linux_rust_bindings::aux::AT_GID as usize;	    /* real gid */
        const AT_EGID =   linux_rust_bindings::aux::AT_EGID as usize;	/* effective gid */
        const AT_PLATFORM = linux_rust_bindings::aux::AT_PLATFORM as usize;  /* string identifying CPU for optimizations */
        const AT_HWCAP =  linux_rust_bindings::aux::AT_HWCAP as usize;   /* arch dependent hints at CPU capabilities */
        const AT_CLKTCK = linux_rust_bindings::aux::AT_CLKTCK as usize;	/* frequency at which times() increments */
        /* AT_* values 18 through 22 are reserved */
        const AT_SECURE = linux_rust_bindings::aux::AT_SECURE as usize;  /* secure mode boolean */
        const AT_BASE_PLATFORM = linux_rust_bindings::aux::AT_BASE_PLATFORM as usize;	/* string identifying real platform, may
                         * differ from AT_PLATFORM. */
        const AT_RANDOM = linux_rust_bindings::aux::AT_RANDOM as usize;	/* address of 16 random bytes */
        const AT_HWCAP2 = linux_rust_bindings::aux::AT_HWCAP2 as usize;	/* extension of AT_HWCAP */

        const AT_EXECFN =  linux_rust_bindings::aux::AT_EXECFN as usize;	/* filename of program */
        const AT_SYSINFO_EHDR = linux_rust_bindings::aux::AT_SYSINFO_EHDR as usize;
        const AT_MINSIGSTKSZ =	linux_rust_bindings::aux::AT_MINSIGSTKSZ as usize;	/* minimal stack size for signal delivery */
    }
}
