/// Some selected aux-values, needs to be kept small since they're collected
/// before symbol relocation on static-pie-linked binaries, which means rustc
/// will emit `memset` on a zeroed allocation of over 256 bytes, which we won't be able
/// to find and thus will result in an immediate segfault on start.
/// See [docs](https://man7.org/linux/man-pages/man3/getauxval.3.html)
#[derive(Debug)]
pub struct AuxValues {
    /// Base address of the program interpreter
    pub at_base: usize,

    /// Real group id of the main thread
    pub at_gid: usize,

    /// Real user id of the main thread
    pub at_uid: usize,

    /// Address of the executable's program headers
    pub at_phdr: usize,

    /// Size of program header entry
    pub at_phent: usize,

    /// Number of program headers
    pub at_phnum: usize,

    /// Address pointing to 16 bytes of a random value
    pub at_random: usize,

    /// Executable should be treated securely
    pub at_secure: usize,

    /// Address of the vdso
    pub at_sysinfo_ehdr: usize,

    /// A pointer to a string containing the pathname used to execute the program
    pub at_execfn: usize,
}

impl AuxValues {
    #[inline(always)]
    #[must_use]
    pub const fn zeroed() -> Self {
        Self {
            at_base: 0,
            at_gid: 0,
            at_uid: 0,
            at_phdr: 0,
            at_phent: 0,
            at_phnum: 0,
            at_random: 0,
            at_secure: 0,
            at_sysinfo_ehdr: 0,
            at_execfn: 0,
        }
    }

    #[must_use]
    #[inline(always)]
    #[expect(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
    pub(crate) unsafe fn from_auxv(auxv: *const usize) -> Self {
        let mut collected = Self::zeroed();
        let mut i = 0;
        let mut key = *auxv;
        while key != 0 {
            if key <= 51 {
                match key as i32 {
                    rusl::platform::AT_PHDR => collected.at_phdr = *(auxv.add(i + 1)),
                    rusl::platform::AT_PHENT => collected.at_phent = *(auxv.add(i + 1)),
                    rusl::platform::AT_PHNUM => collected.at_phnum = *(auxv.add(i + 1)),
                    rusl::platform::AT_BASE => collected.at_base = *(auxv.add(i + 1)),
                    rusl::platform::AT_UID => collected.at_uid = *(auxv.add(i + 1)),
                    rusl::platform::AT_GID => collected.at_gid = *(auxv.add(i + 1)),
                    rusl::platform::AT_SECURE => collected.at_secure = *(auxv.add(i + 1)),
                    rusl::platform::AT_RANDOM => collected.at_random = *(auxv.add(i + 1)),
                    rusl::platform::AT_SYSINFO_EHDR => {
                        collected.at_sysinfo_ehdr = *(auxv.add(i + 1));
                    }
                    rusl::platform::AT_EXECFN => collected.at_execfn = *(auxv.add(i + 1)),
                    _ => {}
                }
            }

            i += 2;
            key = *(auxv.add(i));
        }
        collected
    }
}
