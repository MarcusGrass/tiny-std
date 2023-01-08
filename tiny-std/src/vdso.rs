use core::mem::MaybeUninit;

use rusl::platform::{ElfHeader, ElfSymbol, SectionHeader, TimeSpec};
use rusl::string::unix_str::UnixStr;

const DYNSTR_NAME: &[u8] = b".dynstr\0";
const DYNSYM_NAME: &[u8] = b".dynsym\0";
const CLOCK_GETTIME_NAME: &[u8] = b"__vdso_clock_gettime\0";

/// We want to find the fast way to call `clock_gettime` through the `vdso` if we can.
/// Linux maps the vdso which is a full elf-image into the process memory, we'll use that
/// to see if we can locate the function pointer to `clock_gettime` so that we don't have to
/// go through a syscall each time we want to get the time.
/// To do this we need to do a few things:
/// 1. Find the `.shstrtab` which contain section names (could maybe use section types instead,
/// but .dynstr shares type and attributes with .strtab which could be an issue disambiguating without names)
/// 2. Find the `.dynstr` name offset of the symbol `CLOCK_GETTIME_NAME`, the name offset is in practice like an alias
/// 3. Find the `.dynsym` entry corresponding to the "alias", and get the address offset of the function pointer and the table index of the section containing it
/// 4. Find the required alignment of the containing section by reading its section header
/// 5. Align the offset, transmute to appropriate `extern fn`
/// See [Linux vdso docs](https://man7.org/linux/man-pages/man7/vdso.7.html)
/// See also [Linux elf docs](https://man7.org/linux/man-pages/man5/elf.5.html)
pub(crate) unsafe fn find_vdso_clock_get_time(
    vdso: *const u8,
) -> Option<extern "C" fn(i32, *mut TimeSpec) -> i32> {
    // Elf specifies LE bytes for some fields, this could be an issue
    let mut elf_ptr = MaybeUninit::<ElfHeader>::uninit();
    vdso.copy_to(
        elf_ptr.as_mut_ptr() as *mut u8,
        core::mem::size_of::<ElfHeader>(),
    );
    let header = elf_ptr.assume_init();
    let mut dyn_syms = None;
    // Pointer to the start of the section header
    let section_start = vdso.add(header.0.e_shoff as usize) as *const SectionHeader;
    // Should always be defined, otherwise bail
    let name_section = if header.0.e_shstrndx != 0 {
        section_start.add(header.0.e_shstrndx as usize).read()
    } else {
        return None;
    };

    // Name offset/"alias"
    let mut clock_gettime_st_name_offset: Option<u32> = None;
    // Stop when we've found both `DYNSTR` and `DYNSYM`
    for i in 0..header.0.e_shnum as usize {
        let sect = section_start.add(i).read();
        if match_name(DYNSTR_NAME, &sect, &name_section, vdso) {
            clock_gettime_st_name_offset =
                find_dynstr_st_name_offset_of(CLOCK_GETTIME_NAME, &sect, vdso);
        } else if match_name(DYNSYM_NAME, &sect, &name_section, vdso) {
            dyn_syms = Some(sect);
        }
        if dyn_syms.is_some() && clock_gettime_st_name_offset.is_some() {
            break;
        }
    }
    // Bail if we didn't find the syms
    let dyn_syms = dyn_syms?;
    // Bail if we didn't find the clock st_name_offset
    let clock_alias = clock_gettime_st_name_offset?;
    let function_pointer_info = find_dynsym_ptr_of_name_offset(clock_alias, &dyn_syms, vdso)?;
    // Should be some instruction section, alignment can vary, have found 16
    let containing_section = section_start.add(function_pointer_info.section).read();
    let fptr_align = containing_section.0.sh_addralign as usize;
    let fn_addr = vdso.add(align(function_pointer_info.addr_offset, fptr_align));
    let func: extern "C" fn(i32, *mut TimeSpec) -> i32 = core::mem::transmute(fn_addr);
    Some(func)
}

#[inline]
unsafe fn match_name(
    search_for: &[u8],
    candidate_section: &SectionHeader,
    name_section: &SectionHeader,
    vdso: *const u8,
) -> bool {
    let name_start = candidate_section.0.sh_name as usize;
    let ns_start = name_section.0.sh_offset as usize;
    let ns_ptr = vdso.add(ns_start);
    let start_at = ns_ptr.add(align(name_start, name_section.0.sh_addralign as usize));
    let name = UnixStr::from_ptr(start_at);
    search_for == name.as_slice()
}

#[inline]
unsafe fn find_dynstr_st_name_offset_of(
    search_for: &[u8],
    dyn_str_section: &SectionHeader,
    vdso: *const u8,
) -> Option<u32> {
    // Dynstr starts with a null byte
    let mut offset = 1;
    while offset < dyn_str_section.0.sh_size as usize {
        let start = vdso.add(align(
            dyn_str_section.0.sh_offset as usize + offset,
            dyn_str_section.0.sh_addralign as usize,
        ));
        let first_sym = UnixStr::from_ptr(start);
        if search_for == first_sym.as_slice() {
            return Some(offset as u32);
        }
        offset += first_sym.len();
    }
    None
}

struct FnPtrInfo {
    addr_offset: usize,
    section: usize,
}

#[inline]
unsafe fn find_dynsym_ptr_of_name_offset(
    st_name: u32,
    dynsym: &SectionHeader,
    vdso: *const u8,
) -> Option<FnPtrInfo> {
    let mut offset = 0;
    while offset < dynsym.0.sh_size as usize {
        let search_addr = align(
            dynsym.0.sh_offset as usize + offset,
            dynsym.0.sh_addralign as usize,
        );
        let start = vdso.add(search_addr);
        let mut sym_h = MaybeUninit::<ElfSymbol>::uninit();
        start.copy_to(sym_h.as_mut_ptr() as _, core::mem::size_of::<ElfSymbol>());
        let sym = sym_h.assume_init();
        if sym.0.st_name == st_name {
            // Maybe bail if the type is incorrect, should be `STT_FUNC`, can be found by inspecting
            // `info_to_type` on `st_type`
            return Some(FnPtrInfo {
                addr_offset: sym.0.st_value as usize,
                section: sym.0.st_shndx as usize,
            });
        }
        offset += core::mem::size_of::<ElfSymbol>();
    }
    None
}

#[inline]
fn align(offset: usize, alignment: usize) -> usize {
    offset + (alignment - (offset % alignment)) % alignment
}

#[inline]
#[allow(dead_code)]
fn info_to_type(st_info: u8) -> u32 {
    (st_info as u32) & 0xf
}

#[inline]
#[allow(dead_code)]
fn info_to_bind(st_info: u8) -> u8 {
    st_info >> 4
}
