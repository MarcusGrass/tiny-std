use rusl::platform::{GidT, UidT};

/// A vector of dynamic values supplied by the OS
pub(crate) static mut AUX_VALUES: AuxValues = AuxValues::zeroed();

/// Get the main thread group id
#[inline]
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn get_gid() -> GidT {
    unsafe { AUX_VALUES.at_gid as GidT }
}

/// Get the main thread user id
#[inline]
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn get_uid() -> UidT {
    unsafe { AUX_VALUES.at_uid as UidT }
}

/// Get 16 random bytes of data, run-specific, i.e. does not change between calls.
/// Could call this something like `session-id` to make it more clear that repeated calls
/// won't yield different values.
#[inline]
#[must_use]
pub fn get_random() -> Option<u128> {
    unsafe {
        let random_addr = AUX_VALUES.at_random;
        if random_addr != 0 {
            // This pointer isn't necessarily aligned properly for a u128
            let random_bytes = random_addr as *const u8;
            let unbounded_slice = core::slice::from_raw_parts(random_bytes, 16);
            return Some(u128::from_ne_bytes(
                unbounded_slice.try_into().unwrap_unchecked(),
            ));
        }
        None
    }
}

/// Some selected aux-values, needs to be kept small since they're collected
/// before symbol relocation on static-pie-linked binaries, which means rustc
/// will emit `memset` on a zeroed allocation of over 256 bytes, which we won't be able
/// to find and thus will result in an immediate segfault on start.
/// See [docs](https://man7.org/linux/man-pages/man3/getauxval.3.html)
#[derive(Debug)]
pub(crate) struct AuxValues {
    /// Base address of the program interpreter
    pub(crate) at_base: usize,

    /// Real group id of the main thread
    pub(crate) at_gid: usize,

    /// Real user id of the main thread
    pub(crate) at_uid: usize,

    /// Address of the executable's program headers
    pub(crate) at_phdr: usize,

    /// Size of program header entry
    pub(crate) at_phent: usize,

    /// Number of program headers
    pub(crate) at_phnum: usize,

    /// Address pointing to 16 bytes of a random value
    pub(crate) at_random: usize,

    /// Executable should be treated securely
    pub(crate) at_secure: usize,

    /// Address of the vdso
    pub(crate) at_sysinfo_ehdr: usize,
}

impl AuxValues {
    #[inline(always)]
    pub(crate) const fn zeroed() -> Self {
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
        }
    }

    #[inline(always)]
    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
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
                    _ => {}
                }
            }

            i += 2;
            key = *(auxv.add(i));
        }
        collected
    }
}
