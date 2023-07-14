use core::mem::MaybeUninit;

pub use linux_rust_bindings::elf::*;

#[cfg(target_arch = "x86_64")]
pub const REL_RELATIVE: u64 = 8;
#[cfg(target_arch = "aarch64")]
pub const REL_RELATIVE: u64 = 1027;

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
#[cfg(target_pointer_width = "32")]
pub struct ElfHeader(pub linux_rust_bindings::elf::Elf32_Ehdr);

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
#[cfg(target_pointer_width = "64")]
pub struct ElfHeader(pub linux_rust_bindings::elf::Elf64_Ehdr);

#[repr(C)]
#[derive(Debug)]
pub struct EIdent {
    // Signature
    ei_mag: u32,
    ei_class: u8,
    ei_data: u8,
    ei_version: u8,
    ei_osabi: u8,
    ei_abiversion: u8,
    // On linux we just pad 8 here and skip the rest
    ei_pad: [u8; 7],
}

impl EIdent {
    /// # Safety
    /// Safe if the elf header contains a valid C repr of bytes as `e_ident` ie, the correct
    /// bytes are initialized.
    /// Always safe if all bytes of `e_ident` are initialized, but might give weird results
    /// Todo: Could make always safe by doing a `MaybeUninit::zeroed` afaik
    #[must_use]
    pub unsafe fn from_header(header: ElfHeader) -> Self {
        let mut uninit_self: MaybeUninit<Self> = MaybeUninit::uninit();
        header
            .0
            .e_ident
            .as_ptr()
            .copy_to(uninit_self.as_mut_ptr().cast(), header.0.e_ident.len());
        uninit_self.assume_init()
    }
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
#[cfg(target_pointer_width = "32")]
pub struct SectionHeader(pub linux_rust_bindings::elf::Elf32_Shdr);

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
#[cfg(target_pointer_width = "64")]
pub struct SectionHeader(pub linux_rust_bindings::elf::Elf64_Shdr);

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
#[cfg(target_pointer_width = "32")]
pub struct ElfDynamic(pub linux_rust_bindings::elf::Elf32_Dyn);

#[repr(transparent)]
#[cfg(target_pointer_width = "64")]
pub struct ElfDynamic(pub linux_rust_bindings::elf::Elf64_Dyn);

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
#[cfg(target_pointer_width = "32")]
pub struct ElfSymbol(pub linux_rust_bindings::elf::Elf32_Sym);

#[repr(transparent)]
#[cfg(target_pointer_width = "64")]
pub struct ElfSymbol(pub linux_rust_bindings::elf::Elf64_Sym);

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
#[cfg(target_pointer_width = "64")]
pub struct ElfPhdr(pub linux_rust_bindings::elf::Elf64_Phdr);

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
#[cfg(target_pointer_width = "64")]
pub struct Elf64Rel(pub linux_rust_bindings::elf::Elf64_Rel);

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
#[cfg(target_pointer_width = "64")]
pub struct Elf64Rela(pub linux_rust_bindings::elf::Elf64_Rela);
