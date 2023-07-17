use crate::elf::aux::AuxValues;
use core::hint::unreachable_unchecked;
use rusl::platform::{
    Elf64Rel, Elf64Rela, ElfPhdr, DT_REL, DT_RELA, DT_RELASZ, DT_RELSZ, PT_DYNAMIC, REL_RELATIVE,
};

#[inline(always)]
pub(crate) unsafe fn relocate_symbols(dynv: *const usize, aux: &AuxValues) {
    if dynv as usize != 0 {
        // Static-pie linked (or dynamic but who would do that?)
        // We don't have access to any symbols here, at all.
        let mut base = aux.at_base;
        if base == 0 {
            // Got a direct invocation, ie not external linker, that means we're a static pie
            // and needs to do a relocation
            let ph_num = aux.at_phnum;
            let phent = aux.at_phent;
            let mut phdr_base = aux.at_phdr;
            // Find dynamic entry to adjust the base relocation addr offset
            if ph_num > 0 {
                let mut i = ph_num - 1;
                while i > 0 {
                    let phdr = ptr_unsafe_ref((phdr_base + phent) as *const ElfPhdr);
                    phdr_base += phent;
                    if phdr.0.p_type == PT_DYNAMIC as u32 {
                        base = dynv as usize - phdr.0.p_vaddr as usize;
                        break;
                    }
                    if i == 0 {
                        break;
                    }
                    i -= 1;
                }
            }
            let ds = DynSection::init_from_dynv(dynv);
            ds.relocate(base);
        }
    }
}

pub(crate) struct DynSection {
    rel: usize,
    rel_sz: usize,
    rela: usize,
    rela_sz: usize,
}

impl DynSection {
    /// Function works just like getting aux-values, value is at key + 1
    #[inline(always)]
    pub(crate) unsafe fn init_from_dynv(dynv: *const usize) -> Self {
        let mut ds = Self {
            rel: 0,
            rel_sz: 0,
            rela: 0,
            rela_sz: 0,
        };
        let mut i = 0;
        let mut key = *dynv;
        while key != 0 {
            if key < 19 {
                let ikey = key as i32;
                match ikey {
                    DT_RELA => ds.rela = *(dynv.add(i + 1)),
                    DT_RELASZ => ds.rela_sz = *(dynv.add(i + 1)),
                    DT_REL => ds.rel = *(dynv.add(i + 1)),
                    DT_RELSZ => ds.rel_sz = *(dynv.add(i + 1)),
                    _ => {}
                }
            }

            i += 2;
            key = *(dynv.add(i));
        }
        ds
    }

    #[inline(always)]
    pub(crate) unsafe fn relocate(&self, base_addr: usize) {
        // Relocate all `rel`-entries
        for i in 0..(self.rel_sz / core::mem::size_of::<Elf64Rel>()) {
            let rel_ptr = ((base_addr + self.rel) as *const Elf64Rel).add(i);
            let rel = ptr_unsafe_ref(rel_ptr);
            if rel.0.r_info == relative_type(REL_RELATIVE) {
                let rel_addr = (base_addr + rel.0.r_offset as usize) as *mut usize;
                *rel_addr += base_addr;
            }
        }
        // Relocate all `rela`-entries
        for i in 0..(self.rela_sz / core::mem::size_of::<Elf64Rela>()) {
            let rela_ptr = ((base_addr + self.rela) as *const Elf64Rela).add(i);
            let rela = ptr_unsafe_ref(rela_ptr);
            if rela.0.r_info == relative_type(REL_RELATIVE) {
                let rel_addr = (base_addr + rela.0.r_offset as usize) as *mut usize;
                *rel_addr = base_addr + rela.0.r_addend as usize;
            }
        }
        // Skip implementing `relr`-entries for now
    }
}

#[inline(always)]
const fn relative_type(tp: u64) -> u64 {
    tp & 0x7fffffff
}

#[inline(always)]
unsafe fn ptr_unsafe_ref<T>(ptr: *const T) -> &'static T {
    unwrap_unchecked(ptr.as_ref())
}

#[inline(always)]
unsafe fn unwrap_unchecked<T>(opt: Option<T>) -> T {
    match opt {
        None => unreachable_unchecked(),
        Some(val) => val,
    }
}
