// This is a version of dlmalloc.c ported to Rust. You can find the original
// source at ftp://g.oswego.edu/pub/misc/malloc.c
//
// The original source was written by Doug Lea and released to the public domain
// This exerpt comes from
// https://github.com/alexcrichton/dlmalloc-rs
// Licensed under MIT, license found here:
// https://github.com/alexcrichton/dlmalloc-rs/blob/e72134720f977404b76e113403c46e9758468ef7/LICENSE-MIT
// Copied at https://github.com/alexcrichton/dlmalloc-rs/tree/f352ad5ea6546b95809dc154e9cf195fc740bc3e
// then changed to fit this project.

use core::cmp;
use core::mem;
use core::ptr;
use rusl::platform::is_syscall_error;
use sc::syscall;

#[cfg(feature = "global-allocator")]
#[global_allocator]
static GLOBAL_ALLOC: GlobalDlMalloc = GlobalDlMalloc::new();

#[cfg(all(feature = "global-allocator", feature = "threaded"))]
struct GlobalDlMalloc(crate::sync::Mutex<Dlmalloc>);

#[cfg(all(feature = "global-allocator", feature = "threaded"))]
unsafe impl Sync for GlobalDlMalloc {}

#[cfg(all(feature = "global-allocator", feature = "threaded"))]
unsafe impl Send for GlobalDlMalloc {}

#[cfg(all(feature = "global-allocator", feature = "threaded"))]
impl GlobalDlMalloc {
    const fn new() -> Self {
        Self(crate::sync::Mutex::new(Dlmalloc::new()))
    }
}

#[cfg(all(feature = "global-allocator", feature = "threaded"))]
unsafe impl core::alloc::GlobalAlloc for GlobalDlMalloc {
    #[inline]
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.0.lock().malloc(layout.size(), layout.align())
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
        self.0.lock().free(ptr);
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.0.lock().calloc(layout.size(), layout.align())
    }

    #[inline]
    unsafe fn realloc(
        &self,
        ptr: *mut u8,
        layout: core::alloc::Layout,
        new_size: usize,
    ) -> *mut u8 {
        self.0
            .lock()
            .realloc(ptr, layout.size(), layout.align(), new_size)
    }
}

#[cfg(all(feature = "global-allocator", not(feature = "threaded")))]
struct GlobalDlMalloc;

#[cfg(all(feature = "global-allocator", not(feature = "threaded")))]
impl GlobalDlMalloc {
    const fn new() -> Self {
        Self
    }
}

#[cfg(all(feature = "global-allocator", not(feature = "threaded")))]
static mut ST_DL_MALLOC: Dlmalloc = Dlmalloc::new();

#[cfg(all(feature = "global-allocator", not(feature = "threaded")))]
unsafe impl core::alloc::GlobalAlloc for GlobalDlMalloc {
    #[inline]
    #[expect(static_mut_refs)]
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        ST_DL_MALLOC.malloc(layout.size(), layout.align())
    }

    #[inline]
    #[expect(static_mut_refs)]
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
        ST_DL_MALLOC.free(ptr)
    }

    #[inline]
    #[expect(static_mut_refs)]
    unsafe fn alloc_zeroed(&self, layout: core::alloc::Layout) -> *mut u8 {
        ST_DL_MALLOC.calloc(layout.size(), layout.align())
    }

    #[inline]
    #[expect(static_mut_refs)]
    unsafe fn realloc(
        &self,
        ptr: *mut u8,
        layout: core::alloc::Layout,
        new_size: usize,
    ) -> *mut u8 {
        ST_DL_MALLOC.realloc(ptr, layout.size(), layout.align(), new_size)
    }
}

#[cfg(all(feature = "global-allocator", not(feature = "threaded")))]
unsafe impl Sync for Dlmalloc {}

pub struct Dlmalloc {
    smallmap: u32,
    treemap: u32,
    smallbins: [*mut Chunk; (NSMALLBINS + 1) * 2],
    treebins: [*mut TreeChunk; NTREEBINS],
    dvsize: usize,
    topsize: usize,
    dv: *mut Chunk,
    top: *mut Chunk,
    footprint: usize,
    max_footprint: usize,
    seg: Segment,
    trim_check: usize,
    least_addr: *mut u8,
    release_checks: usize,
}

unsafe impl Send for Dlmalloc {}

// TODO: document this
const NSMALLBINS: usize = 32;
const NTREEBINS: usize = 32;
const SMALLBIN_SHIFT: usize = 3;
const TREEBIN_SHIFT: usize = 8;

// TODO: runtime configurable? documentation?
const DEFAULT_GRANULARITY: usize = 64 * 1024;
const DEFAULT_TRIM_THRESHOLD: usize = 2 * 1024 * 1024;
const MAX_RELEASE_CHECK_RATE: usize = 4095;

const PAGE_SIZE: usize = 4096;

#[repr(C)]
struct Chunk {
    prev_foot: usize,
    head: usize,
    prev: *mut Chunk,
    next: *mut Chunk,
}

#[repr(C)]
struct TreeChunk {
    chunk: Chunk,
    child: [*mut TreeChunk; 2],
    parent: *mut TreeChunk,
    index: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Segment {
    base: *mut u8,
    size: usize,
    next: *mut Segment,
    flags: u32,
}

#[inline]
const fn align_up(a: usize, alignment: usize) -> usize {
    debug_assert!(alignment.is_power_of_two());
    (a + (alignment - 1)) & !(alignment - 1)
}

#[inline]
const fn left_bits(x: u32) -> u32 {
    (x << 1) | (!(x << 1)).wrapping_add(1)
}

#[inline]
const fn least_bit(x: u32) -> u32 {
    x & (!x + 1)
}

#[expect(clippy::cast_possible_truncation)]
const fn leftshift_for_tree_index(x: u32) -> u32 {
    let x = x as usize;
    if x == NTREEBINS - 1 {
        0
    } else {
        (mem::size_of::<usize>() * 8 - 1 - ((x >> 1) + TREEBIN_SHIFT - 2)) as u32
    }
}

#[expect(clippy::new_without_default)]
impl Dlmalloc {
    #[must_use]
    pub const fn new() -> Dlmalloc {
        Dlmalloc {
            smallmap: 0,
            treemap: 0,
            smallbins: [core::ptr::null_mut(); (NSMALLBINS + 1) * 2],
            treebins: [core::ptr::null_mut(); NTREEBINS],
            dvsize: 0,
            topsize: 0,
            dv: core::ptr::null_mut(),
            top: core::ptr::null_mut(),
            footprint: 0,
            max_footprint: 0,
            seg: Segment {
                base: core::ptr::null_mut(),
                size: 0,
                next: core::ptr::null_mut(),
                flags: 0,
            },
            trim_check: 0,
            least_addr: core::ptr::null_mut(),
            release_checks: 0,
        }
    }
    /// Allocates `size` bytes with `align` align.
    ///
    /// Returns a null pointer if allocation fails. Returns a valid pointer
    /// otherwise.
    ///
    /// Safety and contracts are largely governed by the `GlobalAlloc::alloc`
    /// method contracts.
    #[inline]
    #[expect(clippy::missing_safety_doc)]
    pub unsafe fn malloc(&mut self, size: usize, align: usize) -> *mut u8 {
        if align <= Self::MALLOC_ALIGNMENT {
            self.inner_malloc(size)
        } else {
            self.memalign(align, size)
        }
    }

    /// Same as `malloc`, except if the allocation succeeds it's guaranteed to
    /// point to `size` bytes of zeros.
    #[inline]
    #[expect(clippy::missing_safety_doc)]
    pub unsafe fn calloc(&mut self, size: usize, align: usize) -> *mut u8 {
        let ptr = self.malloc(size, align);
        if !ptr.is_null() && Self::calloc_must_clear(ptr) {
            ptr::write_bytes(ptr, 0, size);
        }
        ptr
    }

    /// Reallocates `ptr`, a previous allocation with `old_size` and
    /// `old_align`, to have `new_size` and the same alignment as before.
    ///
    /// Returns a null pointer if the memory couldn't be reallocated, but `ptr`
    /// is still valid. Returns a valid pointer and frees `ptr` if the request
    /// is satisfied.
    ///
    /// Safety and contracts are largely governed by the `GlobalAlloc::realloc`
    /// method contracts.
    #[inline]
    #[expect(clippy::missing_safety_doc)]
    pub unsafe fn realloc(
        &mut self,
        ptr: *mut u8,
        old_size: usize,
        old_align: usize,
        new_size: usize,
    ) -> *mut u8 {
        if old_align <= Self::MALLOC_ALIGNMENT {
            self.inner_realloc(ptr, new_size)
        } else {
            let res = self.malloc(new_size, old_align);
            if !res.is_null() {
                let size = cmp::min(old_size, new_size);
                ptr::copy_nonoverlapping(ptr, res, size);
                self.free(ptr);
            }
            res
        }
    }
}

impl Dlmalloc {
    // TODO: can we get rid of this?
    const MALLOC_ALIGNMENT: usize = mem::size_of::<usize>() * 2;
    const CHUNK_OVERHEAD: usize = mem::size_of::<usize>();
    const MMAP_CHUNK_OVERHEAD: usize = Self::MALLOC_ALIGNMENT;
    const MIN_LARGE_SIZE: usize = 1 << TREEBIN_SHIFT;
    const MAX_SMALL_SIZE: usize = Self::MIN_LARGE_SIZE - 1;
    const MAX_SMALL_REQUEST: usize =
        Self::MAX_SMALL_SIZE - (Self::MALLOC_ALIGNMENT - 1) - Self::CHUNK_OVERHEAD;
    const MIN_CHUNK_SIZE: usize = align_up(mem::size_of::<Chunk>(), Self::MALLOC_ALIGNMENT);
    const MIN_REQUEST: usize = Self::MIN_CHUNK_SIZE - Self::CHUNK_OVERHEAD - 1;
    const MAX_REQUEST: usize = Self::const_max_request();

    const fn const_max_request() -> usize {
        // min_sys_alloc_space: the largest `X` such that
        //   pad_request(X - 1)        -- minus 1, because requests of exactly
        //                                `max_request` will not be honored
        //   + self.top_foot_size()
        //   + Self::MALLOC_ALIGNMENT
        //   + DEFAULT_GRANULARITY
        // ==
        //   usize::MAX
        let min_sys_alloc_space =
            ((!0 - (DEFAULT_GRANULARITY + Self::top_foot_size() + Self::MALLOC_ALIGNMENT) + 1)
                & !Self::MALLOC_ALIGNMENT)
                - Self::CHUNK_OVERHEAD
                + 1;

        let a = (!Self::MIN_CHUNK_SIZE + 1) << 2;
        if a > min_sys_alloc_space {
            min_sys_alloc_space
        } else {
            a
        }
    }

    #[inline]
    const fn pad_request(amt: usize) -> usize {
        align_up(amt + Self::CHUNK_OVERHEAD, Self::MALLOC_ALIGNMENT)
    }

    #[inline]
    #[expect(clippy::cast_possible_truncation)]
    const fn small_index(size: usize) -> u32 {
        (size >> SMALLBIN_SHIFT) as u32
    }

    #[inline]
    const fn small_index2size(idx: u32) -> usize {
        (idx as usize) << SMALLBIN_SHIFT
    }

    #[inline]
    const fn is_small(s: usize) -> bool {
        s >> SMALLBIN_SHIFT < NSMALLBINS
    }

    #[inline]
    const fn is_aligned(a: usize) -> bool {
        a & (Self::MALLOC_ALIGNMENT - 1) == 0
    }

    #[inline]
    fn align_offset(addr: *mut u8) -> usize {
        Self::align_offset_usize(addr as usize)
    }

    #[inline]
    const fn align_offset_usize(addr: usize) -> usize {
        align_up(addr, Self::MALLOC_ALIGNMENT) - (addr)
    }

    const fn top_foot_size() -> usize {
        Self::align_offset_usize(Chunk::MEM_OFFSET)
            + Self::pad_request(mem::size_of::<Segment>())
            + Self::MIN_CHUNK_SIZE
    }

    #[inline]
    fn mmap_foot_pad() -> usize {
        4 * mem::size_of::<usize>()
    }

    fn align_as_chunk(ptr: *mut u8) -> *mut Chunk {
        unsafe {
            let chunk = Chunk::to_mem(ptr.cast());
            ptr.add(Self::align_offset(chunk)).cast()
        }
    }

    const fn request2size(req: usize) -> usize {
        if req < Self::MIN_REQUEST {
            Self::MIN_CHUNK_SIZE
        } else {
            Self::pad_request(req)
        }
    }

    unsafe fn overhead_for(p: *mut Chunk) -> usize {
        if Chunk::mmapped(p) {
            Self::MMAP_CHUNK_OVERHEAD
        } else {
            Self::CHUNK_OVERHEAD
        }
    }

    #[inline]
    unsafe fn calloc_must_clear(ptr: *mut u8) -> bool {
        !Chunk::mmapped(Chunk::from_mem(ptr))
    }

    #[expect(clippy::too_many_lines)]
    unsafe fn inner_malloc(&mut self, size: usize) -> *mut u8 {
        #[cfg(debug_assertions)]
        self.check_malloc_state();

        let nb;
        if size <= Self::MAX_SMALL_REQUEST {
            nb = Self::request2size(size);
            let mut idx = Self::small_index(nb);
            let smallbits = self.smallmap >> idx;

            // Check the bin for `idx` (the lowest bit) but also check the next
            // bin up to use that to satisfy our request, if needed.
            if smallbits & 0b11 != 0 {
                // If our the lowest bit, our `idx`, is unset then bump up the
                // index as we'll be using the next bucket up.
                idx += !smallbits & 1;

                let b = self.smallbin_at(idx);
                let p = (*b).prev;
                self.unlink_first_small_chunk(b, p, idx);
                let smallsize = Self::small_index2size(idx);
                Chunk::set_inuse_and_pinuse(p, smallsize);
                let ret = Chunk::to_mem(p);
                #[cfg(debug_assertions)]
                self.check_malloced_chunk(ret, nb);
                return ret;
            }

            if nb > self.dvsize {
                // If there's some other bin with some memory, then we just use
                // the next smallest bin
                if smallbits != 0 {
                    let leftbits = (smallbits << idx) & left_bits(1 << idx);
                    let leastbit = least_bit(leftbits);
                    let i = leastbit.trailing_zeros();
                    let b = self.smallbin_at(i);
                    let p = (*b).prev;
                    debug_assert_eq!(Chunk::size(p), Self::small_index2size(i));
                    self.unlink_first_small_chunk(b, p, i);
                    let smallsize = Self::small_index2size(i);
                    let rsize = smallsize - nb;
                    if mem::size_of::<usize>() != 4 && rsize < Self::MIN_CHUNK_SIZE {
                        Chunk::set_inuse_and_pinuse(p, smallsize);
                    } else {
                        Chunk::set_size_and_pinuse_of_inuse_chunk(p, nb);
                        let r = Chunk::plus_offset(p, nb);
                        Chunk::set_size_and_pinuse_of_free_chunk(r, rsize);
                        self.replace_dv(r, rsize);
                    }
                    let ret = Chunk::to_mem(p);
                    #[cfg(debug_assertions)]
                    self.check_malloced_chunk(ret, nb);
                    return ret;
                } else if self.treemap != 0 {
                    let mem = self.tmalloc_small(nb);
                    if !mem.is_null() {
                        #[cfg(debug_assertions)]
                        self.check_malloced_chunk(mem, nb);
                        #[cfg(debug_assertions)]
                        self.check_malloc_state();
                        return mem;
                    }
                }
            }
        } else if size >= Self::MAX_REQUEST {
            // TODO: translate this to unsupported
            return ptr::null_mut();
        } else {
            nb = Self::pad_request(size);
            if self.treemap != 0 {
                let mem = self.tmalloc_large(nb);
                if !mem.is_null() {
                    #[cfg(debug_assertions)]
                    self.check_malloced_chunk(mem, nb);
                    #[cfg(debug_assertions)]
                    self.check_malloc_state();
                    return mem;
                }
            }
        }

        // use the `dv` node if we can, splitting it if necessary or otherwise
        // exhausting the entire chunk
        if nb <= self.dvsize {
            let rsize = self.dvsize - nb;
            let p = self.dv;
            if rsize >= Self::MIN_CHUNK_SIZE {
                self.dv = Chunk::plus_offset(p, nb);
                self.dvsize = rsize;
                let r = self.dv;
                Chunk::set_size_and_pinuse_of_free_chunk(r, rsize);
                Chunk::set_size_and_pinuse_of_inuse_chunk(p, nb);
            } else {
                let dvs = self.dvsize;
                self.dvsize = 0;
                self.dv = ptr::null_mut();
                Chunk::set_inuse_and_pinuse(p, dvs);
            }
            let ret = Chunk::to_mem(p);
            #[cfg(debug_assertions)]
            self.check_malloced_chunk(ret, nb);
            #[cfg(debug_assertions)]
            self.check_malloc_state();
            return ret;
        }

        // Split the top node if we can
        if nb < self.topsize {
            self.topsize -= nb;
            let rsize = self.topsize;
            let p = self.top;
            self.top = Chunk::plus_offset(p, nb);
            let r = self.top;
            (*r).head = rsize | PINUSE;
            Chunk::set_size_and_pinuse_of_inuse_chunk(p, nb);
            #[cfg(debug_assertions)]
            self.check_top_chunk(self.top);
            let ret = Chunk::to_mem(p);
            #[cfg(debug_assertions)]
            self.check_malloced_chunk(ret, nb);
            #[cfg(debug_assertions)]
            self.check_malloc_state();
            return ret;
        }

        self.sys_alloc(nb)
    }

    /// allocates system resources
    unsafe fn sys_alloc(&mut self, size: usize) -> *mut u8 {
        #[cfg(debug_assertions)]
        self.check_malloc_state();
        // keep in sync with max_request
        let asize = align_up(
            size + Self::top_foot_size() + Self::MALLOC_ALIGNMENT,
            DEFAULT_GRANULARITY,
        );

        let (tbase, tsize, flags) = syscall_alloc(asize);
        if tbase.is_null() {
            return tbase;
        }

        self.footprint += tsize;
        self.max_footprint = cmp::max(self.max_footprint, self.footprint);

        if self.top.is_null() {
            if self.least_addr.is_null() || tbase < self.least_addr {
                self.least_addr = tbase;
            }
            self.seg.base = tbase;
            self.seg.size = tsize;
            self.seg.flags = flags;
            self.release_checks = MAX_RELEASE_CHECK_RATE;
            self.init_bins();
            let tsize = tsize - Self::top_foot_size();
            self.init_top(tbase.cast(), tsize);
        } else {
            let mut sp = ptr::addr_of_mut!(self.seg);
            while !sp.is_null() && tbase != Segment::top(sp) {
                sp = (*sp).next;
            }
            if !sp.is_null()
                && Segment::sys_flags(sp) == flags
                && Segment::holds(sp, self.top.cast())
            {
                (*sp).size += tsize;
                let ptr = self.top;
                let size = self.topsize + tsize;
                self.init_top(ptr, size);
            } else {
                self.least_addr = cmp::min(tbase, self.least_addr);
                let mut sp = ptr::addr_of_mut!(self.seg);
                while !sp.is_null() && (*sp).base != tbase.add(tsize) {
                    sp = (*sp).next;
                }
                if !sp.is_null() && Segment::sys_flags(sp) == flags {
                    let oldbase = (*sp).base;
                    (*sp).base = tbase;
                    (*sp).size += tsize;
                    return self.prepend_alloc(tbase, oldbase, size);
                }
                self.add_segment(tbase, tsize, flags);
            }
        }

        if size < self.topsize {
            self.topsize -= size;
            let rsize = self.topsize;
            let p = self.top;
            self.top = Chunk::plus_offset(p, size);
            let r = self.top;
            (*r).head = rsize | PINUSE;
            Chunk::set_size_and_pinuse_of_inuse_chunk(p, size);
            let ret = Chunk::to_mem(p);
            #[cfg(debug_assertions)]
            self.check_top_chunk(self.top);
            #[cfg(debug_assertions)]
            self.check_malloced_chunk(ret, size);
            #[cfg(debug_assertions)]
            self.check_malloc_state();
            return ret;
        }

        ptr::null_mut()
    }

    unsafe fn inner_realloc(&mut self, oldmem: *mut u8, bytes: usize) -> *mut u8 {
        if bytes >= Self::MAX_REQUEST {
            return ptr::null_mut();
        }
        let nb = Self::request2size(bytes);
        let oldp = Chunk::from_mem(oldmem);
        let newp = self.try_realloc_chunk(oldp, nb, true);
        if !newp.is_null() {
            #[cfg(debug_assertions)]
            self.check_inuse_chunk(newp);
            return Chunk::to_mem(newp);
        }
        let ptr = self.inner_malloc(bytes);
        if !ptr.is_null() {
            let oc = Chunk::size(oldp) - Self::overhead_for(oldp);
            ptr::copy_nonoverlapping(oldmem, ptr, cmp::min(oc, bytes));
            self.free(oldmem);
        }
        ptr
    }

    unsafe fn try_realloc_chunk(&mut self, p: *mut Chunk, nb: usize, can_move: bool) -> *mut Chunk {
        let oldsize = Chunk::size(p);
        let next = Chunk::plus_offset(p, oldsize);

        if Chunk::mmapped(p) {
            self.mmap_resize(p, nb, can_move)
        } else if oldsize >= nb {
            let rsize = oldsize - nb;
            if rsize >= Self::MIN_CHUNK_SIZE {
                let r = Chunk::plus_offset(p, nb);
                Chunk::set_inuse(p, nb);
                Chunk::set_inuse(r, rsize);
                self.dispose_chunk(r, rsize);
            }
            p
        } else if next == self.top {
            // extend into top
            if oldsize + self.topsize <= nb {
                return ptr::null_mut();
            }
            let newsize = oldsize + self.topsize;
            let newtopsize = newsize - nb;
            let newtop = Chunk::plus_offset(p, nb);
            Chunk::set_inuse(p, nb);
            (*newtop).head = newtopsize | PINUSE;
            self.top = newtop;
            self.topsize = newtopsize;
            p
        } else if next == self.dv {
            // extend into dv
            let dvs = self.dvsize;
            if oldsize + dvs < nb {
                return ptr::null_mut();
            }
            let dsize = oldsize + dvs - nb;
            if dsize >= Self::MIN_CHUNK_SIZE {
                let r = Chunk::plus_offset(p, nb);
                let n = Chunk::plus_offset(r, dsize);
                Chunk::set_inuse(p, nb);
                Chunk::set_size_and_pinuse_of_free_chunk(r, dsize);
                Chunk::clear_pinuse(n);
                self.dvsize = dsize;
                self.dv = r;
            } else {
                // exhaust dv
                let newsize = oldsize + dvs;
                Chunk::set_inuse(p, newsize);
                self.dvsize = 0;
                self.dv = ptr::null_mut();
            }
            p
        } else if !Chunk::cinuse(next) {
            // extend into the next free chunk
            let nextsize = Chunk::size(next);
            if oldsize + nextsize < nb {
                return ptr::null_mut();
            }
            let rsize = oldsize + nextsize - nb;
            self.unlink_chunk(next, nextsize);
            if rsize < Self::MIN_CHUNK_SIZE {
                let newsize = oldsize + nextsize;
                Chunk::set_inuse(p, newsize);
            } else {
                let r = Chunk::plus_offset(p, nb);
                Chunk::set_inuse(p, nb);
                Chunk::set_inuse(r, rsize);
                self.dispose_chunk(r, rsize);
            }
            p
        } else {
            ptr::null_mut()
        }
    }

    #[expect(clippy::cast_ptr_alignment)]
    unsafe fn mmap_resize(&mut self, oldp: *mut Chunk, nb: usize, can_move: bool) -> *mut Chunk {
        let oldsize = Chunk::size(oldp);
        // Can't shrink mmap regions below a small size
        if Self::is_small(nb) {
            return ptr::null_mut();
        }

        // Keep the old chunk if it's big enough but not too big
        if oldsize >= nb + mem::size_of::<usize>() && (oldsize - nb) <= (DEFAULT_GRANULARITY << 1) {
            return oldp;
        }

        let offset = (*oldp).prev_foot;
        let oldmmsize = oldsize + offset + Self::mmap_foot_pad();
        let newmmsize =
            Self::mmap_align(nb + 6 * mem::size_of::<usize>() + Self::MALLOC_ALIGNMENT - 1);
        let ptr = syscall_remap(
            (oldp.cast::<u8>()).sub(offset),
            oldmmsize,
            newmmsize,
            can_move,
        );
        if ptr.is_null() {
            return ptr::null_mut();
        }
        let newp = ptr.add(offset).cast::<Chunk>();
        let psize = newmmsize - offset - Self::mmap_foot_pad();
        (*newp).head = psize;
        (*Chunk::plus_offset(newp, psize)).head = Chunk::FENCEPOST_HEAD;
        (*Chunk::plus_offset(newp, psize + mem::size_of::<usize>())).head = 0;
        self.least_addr = cmp::min(ptr, self.least_addr);
        self.footprint = self.footprint + newmmsize - oldmmsize;
        self.max_footprint = cmp::max(self.max_footprint, self.footprint);
        #[cfg(debug_assertions)]
        self.check_mmapped_chunk(newp);
        newp
    }

    #[inline]
    const fn mmap_align(a: usize) -> usize {
        align_up(a, PAGE_SIZE)
    }

    // Only call this with power-of-two alignment and alignment >
    // `Self::MALLOC_ALIGNMENT`
    #[expect(clippy::cast_ptr_alignment)]
    unsafe fn memalign(&mut self, mut alignment: usize, bytes: usize) -> *mut u8 {
        if alignment < Self::MIN_CHUNK_SIZE {
            alignment = Self::MIN_CHUNK_SIZE;
        }
        if bytes >= Self::MAX_REQUEST - alignment {
            return ptr::null_mut();
        }
        let nb = Self::request2size(bytes);
        let req = nb + alignment + Self::MIN_CHUNK_SIZE - Self::CHUNK_OVERHEAD;
        let mem = self.inner_malloc(req);
        if mem.is_null() {
            return mem;
        }
        let mut p = Chunk::from_mem(mem);
        if mem as usize & (alignment - 1) != 0 {
            // Here we find an aligned sopt inside the chunk. Since we need to
            // give back leading space in a chunk of at least `min_chunk_size`,
            // if the first calculation places us at a spot with less than
            // `min_chunk_size` leader we can move to the next aligned spot.
            // we've allocated enough total room so that this is always possible
            let br =
                Chunk::from_mem(((mem as usize + alignment - 1) & (!alignment + 1)) as *mut u8);
            let pos = if (br as usize - p as usize) > Self::MIN_CHUNK_SIZE {
                br.cast::<u8>()
            } else {
                (br.cast::<u8>()).add(alignment)
            };
            let newp = pos.cast::<Chunk>();
            let leadsize = pos as usize - p as usize;
            let newsize = Chunk::size(p) - leadsize;

            // for mmapped chunks just adjust the offset
            if Chunk::mmapped(p) {
                (*newp).prev_foot = (*p).prev_foot + leadsize;
                (*newp).head = newsize;
            } else {
                // give back the leader, use the rest
                Chunk::set_inuse(newp, newsize);
                Chunk::set_inuse(p, leadsize);
                self.dispose_chunk(p, leadsize);
            }
            p = newp;
        }

        // give back spare room at the end
        if !Chunk::mmapped(p) {
            let size = Chunk::size(p);
            if size > nb + Self::MIN_CHUNK_SIZE {
                let remainder_size = size - nb;
                let remainder = Chunk::plus_offset(p, nb);
                Chunk::set_inuse(p, nb);
                Chunk::set_inuse(remainder, remainder_size);
                self.dispose_chunk(remainder, remainder_size);
            }
        }

        let mem = Chunk::to_mem(p);
        debug_assert!(Chunk::size(p) >= nb);
        debug_assert_eq!(align_up(mem as usize, alignment), mem as usize);
        #[cfg(debug_assertions)]
        self.check_inuse_chunk(p);
        mem
    }

    // consolidate and bin a chunk, differs from exported versions of free
    // mainly in that the chunk need not be marked as inuse
    unsafe fn dispose_chunk(&mut self, mut p: *mut Chunk, mut psize: usize) {
        let next = Chunk::plus_offset(p, psize);
        if !Chunk::pinuse(p) {
            let prevsize = (*p).prev_foot;
            if Chunk::mmapped(p) {
                psize += prevsize + Self::mmap_foot_pad();
                if syscall_free((p.cast::<u8>()).sub(prevsize), psize) {
                    self.footprint -= psize;
                }
                return;
            }
            let prev = Chunk::minus_offset(p, prevsize);
            psize += prevsize;
            p = prev;
            if p != self.dv {
                self.unlink_chunk(p, prevsize);
            } else if (*next).head & INUSE == INUSE {
                self.dvsize = psize;
                Chunk::set_free_with_pinuse(p, psize, next);
                return;
            }
        }

        if Chunk::cinuse(next) {
            Chunk::set_free_with_pinuse(p, psize, next);
        } else {
            // consolidate forward
            if next == self.top {
                self.topsize += psize;
                let tsize = self.topsize;
                self.top = p;
                (*p).head = tsize | PINUSE;
                if p == self.dv {
                    self.dv = ptr::null_mut();
                    self.dvsize = 0;
                }
                return;
            } else if next == self.dv {
                self.dvsize += psize;
                let dsize = self.dvsize;
                self.dv = p;
                Chunk::set_size_and_pinuse_of_free_chunk(p, dsize);
                return;
            }
            let nsize = Chunk::size(next);
            psize += nsize;
            self.unlink_chunk(next, nsize);
            Chunk::set_size_and_pinuse_of_free_chunk(p, psize);
            if p == self.dv {
                self.dvsize = psize;
                return;
            }
        }
        self.insert_chunk(p, psize);
    }

    unsafe fn init_top(&mut self, ptr: *mut Chunk, size: usize) {
        let offset = Self::align_offset(Chunk::to_mem(ptr));
        let p = Chunk::plus_offset(ptr, offset);
        let size = size - offset;

        self.top = p;
        self.topsize = size;
        (*p).head = size | PINUSE;
        (*Chunk::plus_offset(p, size)).head = Self::top_foot_size();
        self.trim_check = DEFAULT_TRIM_THRESHOLD;
    }

    #[expect(clippy::cast_possible_truncation)]
    unsafe fn init_bins(&mut self) {
        for i in 0..NSMALLBINS as u32 {
            let bin = self.smallbin_at(i);
            (*bin).next = bin;
            (*bin).prev = bin;
        }
    }

    unsafe fn prepend_alloc(&mut self, newbase: *mut u8, oldbase: *mut u8, size: usize) -> *mut u8 {
        let p = Self::align_as_chunk(newbase);
        let mut oldfirst = Self::align_as_chunk(oldbase);
        let psize = oldfirst as usize - p as usize;
        let q = Chunk::plus_offset(p, size);
        let mut qsize = psize - size;
        Chunk::set_size_and_pinuse_of_inuse_chunk(p, size);

        debug_assert!(oldfirst > q);
        debug_assert!(Chunk::pinuse(oldfirst));
        debug_assert!(qsize >= Self::MIN_CHUNK_SIZE);

        // consolidate the remainder with the first chunk of the old base
        if oldfirst == self.top {
            self.topsize += qsize;
            let tsize = self.topsize;
            self.top = q;
            (*q).head = tsize | PINUSE;
            #[cfg(debug_assertions)]
            self.check_top_chunk(q);
        } else if oldfirst == self.dv {
            self.dvsize += qsize;
            let dsize = self.dvsize;
            self.dv = q;
            Chunk::set_size_and_pinuse_of_free_chunk(q, dsize);
        } else {
            if !Chunk::inuse(oldfirst) {
                let nsize = Chunk::size(oldfirst);
                self.unlink_chunk(oldfirst, nsize);
                oldfirst = Chunk::plus_offset(oldfirst, nsize);
                qsize += nsize;
            }
            Chunk::set_free_with_pinuse(q, qsize, oldfirst);
            self.insert_chunk(q, qsize);
            #[cfg(debug_assertions)]
            self.check_free_chunk(q);
        }

        let ret = Chunk::to_mem(p);
        #[cfg(debug_assertions)]
        self.check_malloced_chunk(ret, size);
        #[cfg(debug_assertions)]
        self.check_malloc_state();
        ret
    }

    // add a segment to hold a new noncontiguous region
    #[expect(clippy::cast_ptr_alignment)]
    #[expect(clippy::similar_names)]
    unsafe fn add_segment(&mut self, tbase: *mut u8, tsize: usize, flags: u32) {
        // TODO: what in the world is this function doing

        // Determine locations and sizes of segment, fenceposts, and the old top
        let old_top = self.top.cast::<u8>();
        let oldsp = self.segment_holding(old_top);
        let old_end = Segment::top(oldsp);
        let ssize = Self::pad_request(mem::size_of::<Segment>());
        let offset = ssize + mem::size_of::<usize>() * 4 + Self::MALLOC_ALIGNMENT - 1;
        let rawsp = old_end.sub(offset);
        let offset = Self::align_offset(Chunk::to_mem(rawsp.cast()));
        let asp = rawsp.add(offset);
        let csp = if asp < old_top.add(Self::MIN_CHUNK_SIZE) {
            old_top
        } else {
            asp
        };
        let sp = csp.cast();
        let ss = Chunk::to_mem(sp).cast();
        let tnext = Chunk::plus_offset(sp, ssize);
        let mut p = tnext;
        let mut nfences = 0;

        // reset the top to our new space
        let size = tsize - Self::top_foot_size();
        self.init_top(tbase.cast(), size);

        // set up our segment record
        debug_assert!(Self::is_aligned(ss as usize));
        Chunk::set_size_and_pinuse_of_inuse_chunk(sp, ssize);
        *ss = self.seg; // push our current record
        self.seg.base = tbase;
        self.seg.size = tsize;
        self.seg.flags = flags;
        self.seg.next = ss;

        // insert trailing fences
        loop {
            let nextp = Chunk::plus_offset(p, mem::size_of::<usize>());
            (*p).head = Chunk::FENCEPOST_HEAD;
            nfences += 1;
            let addr = ptr::addr_of!((*nextp).head);
            if (addr as *mut u8) < old_end {
                p = nextp;
            } else {
                break;
            }
        }
        debug_assert!(nfences >= 2);

        // insert the rest of the old top into a bin as an ordinary free chunk
        if csp != old_top {
            let q = old_top.cast::<Chunk>();
            let psize = csp as usize - old_top as usize;
            let tn = Chunk::plus_offset(q, psize);
            Chunk::set_free_with_pinuse(q, psize, tn);
            self.insert_chunk(q, psize);
        }

        #[cfg(debug_assertions)]
        self.check_top_chunk(self.top);
        #[cfg(debug_assertions)]
        self.check_malloc_state();
    }

    unsafe fn segment_holding(&self, ptr: *mut u8) -> *mut Segment {
        let sp = ptr::addr_of!(self.seg);
        let mut sp = sp.cast_mut();
        while !sp.is_null() {
            if (*sp).base <= ptr && ptr < Segment::top(sp) {
                return sp;
            }
            sp = (*sp).next;
        }
        ptr::null_mut()
    }

    unsafe fn tmalloc_small(&mut self, size: usize) -> *mut u8 {
        let leastbit = least_bit(self.treemap);
        let i = leastbit.trailing_zeros();
        let mut v = *self.treebin_at(i);
        let mut t = v;
        let mut rsize = Chunk::size(TreeChunk::chunk(t)) - size;

        loop {
            t = TreeChunk::leftmost_child(t);
            if t.is_null() {
                break;
            }
            let trem = Chunk::size(TreeChunk::chunk(t)) - size;
            if trem < rsize {
                rsize = trem;
                v = t;
            }
        }

        let vc = TreeChunk::chunk(v);
        let r = Chunk::plus_offset(vc, size).cast::<TreeChunk>();
        debug_assert_eq!(Chunk::size(vc), rsize + size);
        self.unlink_large_chunk(v);
        if rsize < Self::MIN_CHUNK_SIZE {
            Chunk::set_inuse_and_pinuse(vc, rsize + size);
        } else {
            let rc = TreeChunk::chunk(r);
            Chunk::set_size_and_pinuse_of_inuse_chunk(vc, size);
            Chunk::set_size_and_pinuse_of_free_chunk(rc, rsize);
            self.replace_dv(rc, rsize);
        }
        Chunk::to_mem(vc)
    }

    unsafe fn tmalloc_large(&mut self, size: usize) -> *mut u8 {
        let mut v = ptr::null_mut();
        let mut rsize = !size + 1;
        let idx = Self::compute_tree_index(size);
        let mut t = *self.treebin_at(idx);
        if !t.is_null() {
            // Traverse thre tree for this bin looking for a node with size
            // equal to the `size` above.
            let mut sizebits = size << leftshift_for_tree_index(idx);
            // Keep track of the deepest untaken right subtree
            let mut rst = ptr::null_mut();
            loop {
                let csize = Chunk::size(TreeChunk::chunk(t));
                if csize >= size && csize - size < rsize {
                    v = t;
                    rsize = csize - size;
                    if rsize == 0 {
                        break;
                    }
                }
                let rt = (*t).child[1];
                t = (*t).child[(sizebits >> (mem::size_of::<usize>() * 8 - 1)) & 1];
                if !rt.is_null() && rt != t {
                    rst = rt;
                }
                if t.is_null() {
                    // Reset `t` to the least subtree holding sizes greater than
                    // the `size` above, breaking out
                    t = rst;
                    break;
                }
                sizebits <<= 1;
            }
        }

        // Set t to the root of the next non-empty treebin
        if t.is_null() && v.is_null() {
            let leftbits = left_bits(1 << idx) & self.treemap;
            if leftbits != 0 {
                let leastbit = least_bit(leftbits);
                let i = leastbit.trailing_zeros();
                t = *self.treebin_at(i);
            }
        }

        // Find the smallest of this tree or subtree
        while !t.is_null() {
            let csize = Chunk::size(TreeChunk::chunk(t));
            if csize >= size && csize - size < rsize {
                rsize = csize - size;
                v = t;
            }
            t = TreeChunk::leftmost_child(t);
        }

        // If dv is a better fit, then return null so malloc will use it
        if v.is_null() || (self.dvsize >= size && rsize >= self.dvsize - size) {
            return ptr::null_mut();
        }

        let vc = TreeChunk::chunk(v);
        let r = Chunk::plus_offset(vc, size);
        debug_assert_eq!(Chunk::size(vc), rsize + size);
        self.unlink_large_chunk(v);
        if rsize < Self::MIN_CHUNK_SIZE {
            Chunk::set_inuse_and_pinuse(vc, rsize + size);
        } else {
            Chunk::set_size_and_pinuse_of_inuse_chunk(vc, size);
            Chunk::set_size_and_pinuse_of_free_chunk(r, rsize);
            self.insert_chunk(r, rsize);
        }
        Chunk::to_mem(vc)
    }

    #[inline]
    unsafe fn smallbin_at(&mut self, idx: u32) -> *mut Chunk {
        debug_assert!(((idx * 2) as usize) < self.smallbins.len());
        let unchecked = self.smallbins.get_unchecked_mut((idx as usize) * 2);
        let p = ptr::addr_of_mut!(*unchecked);
        p.cast()
    }

    #[inline]
    unsafe fn treebin_at(&mut self, idx: u32) -> *mut *mut TreeChunk {
        debug_assert!((idx as usize) < self.treebins.len());
        &raw mut *self.treebins.get_unchecked_mut(idx as usize)
    }

    #[expect(clippy::cast_possible_truncation)]
    fn compute_tree_index(size: usize) -> u32 {
        let x = size >> TREEBIN_SHIFT;
        if x == 0 {
            0
        } else if x > 0xffff {
            NTREEBINS as u32 - 1
        } else {
            let k = mem::size_of_val(&x) * 8 - 1 - (x.leading_zeros() as usize);
            ((k << 1) + (size >> (k + TREEBIN_SHIFT - 1) & 1)) as u32
        }
    }

    unsafe fn unlink_first_small_chunk(&mut self, head: *mut Chunk, next: *mut Chunk, idx: u32) {
        let ptr = (*next).prev;
        debug_assert!(next != head);
        debug_assert!(next != ptr);
        debug_assert_eq!(Chunk::size(next), Self::small_index2size(idx));
        if head == ptr {
            self.clear_smallmap(idx);
        } else {
            (*ptr).next = head;
            (*head).prev = ptr;
        }
    }

    unsafe fn replace_dv(&mut self, chunk: *mut Chunk, size: usize) {
        let dvs = self.dvsize;
        debug_assert!(Self::is_small(dvs));
        if dvs != 0 {
            let dv = self.dv;
            self.insert_small_chunk(dv, dvs);
        }
        self.dvsize = size;
        self.dv = chunk;
    }

    unsafe fn insert_chunk(&mut self, chunk: *mut Chunk, size: usize) {
        if Self::is_small(size) {
            self.insert_small_chunk(chunk, size);
        } else {
            self.insert_large_chunk(chunk.cast(), size);
        }
    }

    unsafe fn insert_small_chunk(&mut self, chunk: *mut Chunk, size: usize) {
        let idx = Self::small_index(size);
        let head = self.smallbin_at(idx);
        let mut f = head;
        debug_assert!(size >= Self::MIN_CHUNK_SIZE);
        if self.smallmap_is_marked(idx) {
            f = (*head).prev;
        } else {
            self.mark_smallmap(idx);
        }

        (*head).prev = chunk;
        (*f).next = chunk;
        (*chunk).prev = f;
        (*chunk).next = head;
    }

    unsafe fn insert_large_chunk(&mut self, chunk: *mut TreeChunk, size: usize) {
        let idx = Self::compute_tree_index(size);
        let h = self.treebin_at(idx);
        (*chunk).index = idx;
        (*chunk).child[0] = ptr::null_mut();
        (*chunk).child[1] = ptr::null_mut();
        let chunkc = TreeChunk::chunk(chunk);
        if self.treemap_is_marked(idx) {
            let mut t = *h;
            let mut k = size << leftshift_for_tree_index(idx);
            loop {
                if Chunk::size(TreeChunk::chunk(t)) == size {
                    let tc = TreeChunk::chunk(t);
                    let f = (*tc).prev;
                    (*f).next = chunkc;
                    (*tc).prev = chunkc;
                    (*chunkc).prev = f;
                    (*chunkc).next = tc;
                    (*chunk).parent = ptr::null_mut();
                    break;
                }
                let c = &mut (*t).child[(k >> (mem::size_of::<usize>() * 8 - 1)) & 1];
                k <<= 1;
                if c.is_null() {
                    *c = chunk;
                    (*chunk).parent = t;
                    (*chunkc).next = chunkc;
                    (*chunkc).prev = chunkc;
                    break;
                }
                t = *c;
            }
        } else {
            self.mark_treemap(idx);
            *h = chunk;
            (*chunk).parent = h.cast::<TreeChunk>(); // TODO: dubious?
            (*chunkc).next = chunkc;
            (*chunkc).prev = chunkc;
        }
    }

    unsafe fn smallmap_is_marked(&self, idx: u32) -> bool {
        self.smallmap & (1 << idx) != 0
    }

    unsafe fn mark_smallmap(&mut self, idx: u32) {
        self.smallmap |= 1 << idx;
    }

    unsafe fn clear_smallmap(&mut self, idx: u32) {
        self.smallmap &= !(1 << idx);
    }

    unsafe fn treemap_is_marked(&self, idx: u32) -> bool {
        self.treemap & (1 << idx) != 0
    }

    unsafe fn mark_treemap(&mut self, idx: u32) {
        self.treemap |= 1 << idx;
    }

    unsafe fn clear_treemap(&mut self, idx: u32) {
        self.treemap &= !(1 << idx);
    }

    unsafe fn unlink_chunk(&mut self, chunk: *mut Chunk, size: usize) {
        if Self::is_small(size) {
            self.unlink_small_chunk(chunk, size);
        } else {
            self.unlink_large_chunk(chunk.cast());
        }
    }

    unsafe fn unlink_small_chunk(&mut self, chunk: *mut Chunk, size: usize) {
        let f = (*chunk).prev;
        let b = (*chunk).next;
        let idx = Self::small_index(size);
        debug_assert!(chunk != b);
        debug_assert!(chunk != f);
        debug_assert_eq!(Chunk::size(chunk), Self::small_index2size(idx));
        if b == f {
            self.clear_smallmap(idx);
        } else {
            (*f).next = b;
            (*b).prev = f;
        }
    }

    unsafe fn unlink_large_chunk(&mut self, chunk: *mut TreeChunk) {
        let xp = (*chunk).parent;
        let mut r;
        if TreeChunk::next(chunk) == chunk {
            let mut rp = &mut (*chunk).child[1];
            if rp.is_null() {
                rp = &mut (*chunk).child[0];
            }
            r = *rp;
            if !rp.is_null() {
                loop {
                    let mut cp = &mut (**rp).child[1];
                    if cp.is_null() {
                        cp = &mut (**rp).child[0];
                    }
                    if cp.is_null() {
                        break;
                    }
                    rp = cp;
                }
                r = *rp;
                *rp = ptr::null_mut();
            }
        } else {
            let f = TreeChunk::prev(chunk);
            r = TreeChunk::next(chunk);
            (*f).chunk.next = TreeChunk::chunk(r);
            (*r).chunk.prev = TreeChunk::chunk(f);
        }

        if xp.is_null() {
            return;
        }

        let h = self.treebin_at((*chunk).index);
        if chunk == *h {
            *h = r;
            if r.is_null() {
                self.clear_treemap((*chunk).index);
            }
        } else if (*xp).child[0] == chunk {
            (*xp).child[0] = r;
        } else {
            (*xp).child[1] = r;
        }

        if !r.is_null() {
            (*r).parent = xp;
            let c0 = (*chunk).child[0];
            if !c0.is_null() {
                (*r).child[0] = c0;
                (*c0).parent = r;
            }
            let c1 = (*chunk).child[1];
            if !c1.is_null() {
                (*r).child[1] = c1;
                (*c1).parent = r;
            }
        }
    }

    #[expect(clippy::missing_safety_doc)]
    pub unsafe fn free(&mut self, mem: *mut u8) {
        #[cfg(debug_assertions)]
        self.check_malloc_state();

        let mut p = Chunk::from_mem(mem);
        let mut psize = Chunk::size(p);
        let next = Chunk::plus_offset(p, psize);
        if !Chunk::pinuse(p) {
            let prevsize = (*p).prev_foot;

            if Chunk::mmapped(p) {
                psize += prevsize + Self::mmap_foot_pad();
                if syscall_free((p.cast::<u8>()).sub(prevsize), psize) {
                    self.footprint -= psize;
                }
                return;
            }

            let prev = Chunk::minus_offset(p, prevsize);
            psize += prevsize;
            p = prev;
            if p != self.dv {
                self.unlink_chunk(p, prevsize);
            } else if (*next).head & INUSE == INUSE {
                self.dvsize = psize;
                Chunk::set_free_with_pinuse(p, psize, next);
                return;
            }
        }

        // Consolidate forward if we can
        if Chunk::cinuse(next) {
            Chunk::set_free_with_pinuse(p, psize, next);
        } else if next == self.top {
            self.topsize += psize;
            let tsize = self.topsize;
            self.top = p;
            (*p).head = tsize | PINUSE;
            if p == self.dv {
                self.dv = ptr::null_mut();
                self.dvsize = 0;
            }
            if self.should_trim(tsize) {
                self.sys_trim(0);
            }
            return;
        } else if next == self.dv {
            self.dvsize += psize;
            let dsize = self.dvsize;
            self.dv = p;
            Chunk::set_size_and_pinuse_of_free_chunk(p, dsize);
            return;
        } else {
            let nsize = Chunk::size(next);
            psize += nsize;
            self.unlink_chunk(next, nsize);
            Chunk::set_size_and_pinuse_of_free_chunk(p, psize);
            if p == self.dv {
                self.dvsize = psize;
                return;
            }
        }

        if Self::is_small(psize) {
            self.insert_small_chunk(p, psize);
            #[cfg(debug_assertions)]
            self.check_free_chunk(p);
        } else {
            self.insert_large_chunk(p.cast(), psize);
            #[cfg(debug_assertions)]
            self.check_free_chunk(p);
            self.release_checks -= 1;
            if self.release_checks == 0 {
                self.release_unused_segments();
            }
        }
    }

    fn should_trim(&self, size: usize) -> bool {
        size > self.trim_check
    }

    #[expect(clippy::manual_div_ceil)]
    unsafe fn sys_trim(&mut self, mut pad: usize) -> bool {
        let mut released = 0;
        if pad < Self::MAX_REQUEST && !self.top.is_null() {
            pad += Self::top_foot_size();
            if self.topsize > pad {
                let unit = DEFAULT_GRANULARITY;
                let extra = ((self.topsize - pad + unit - 1) / unit - 1) * unit;
                let sp = self.segment_holding(self.top.cast());
                debug_assert!(!sp.is_null());

                if (*sp).size >= extra && !self.has_segment_link(sp) {
                    let newsize = (*sp).size - extra;
                    if syscall_free_part((*sp).base, (*sp).size, newsize) {
                        released = extra;
                    }
                }

                if released != 0 {
                    (*sp).size -= released;
                    self.footprint -= released;
                    let top = self.top;
                    let topsize = self.topsize - released;
                    self.init_top(top, topsize);
                    #[cfg(debug_assertions)]
                    self.check_top_chunk(self.top);
                }
            }

            released += self.release_unused_segments();

            if released == 0 && self.topsize > self.trim_check {
                self.trim_check = usize::MAX;
            }
        }

        released != 0
    }

    unsafe fn has_segment_link(&self, ptr: *mut Segment) -> bool {
        let sp = ptr::addr_of!(self.seg);
        let mut sp = sp.cast_mut();
        while !sp.is_null() {
            if Segment::holds(ptr, sp.cast::<u8>()) {
                return true;
            }
            sp = (*sp).next;
        }
        false
    }

    /// Unmap and unlink any mapped segments that don't contain used chunks
    unsafe fn release_unused_segments(&mut self) -> usize {
        let mut released = 0;
        let mut nsegs = 0;
        let mut pred = ptr::addr_of_mut!(self.seg);
        let mut sp = (*pred).next;
        while !sp.is_null() {
            let base = (*sp).base;
            let size = (*sp).size;
            let next = (*sp).next;
            nsegs += 1;

            let p = Self::align_as_chunk(base);
            let psize = Chunk::size(p);
            // We can unmap if the first chunk holds the entire segment and
            // isn't pinned.
            let chunk_top = (p.cast::<u8>()).add(psize);
            let top = base.add(size - Self::top_foot_size());
            if !Chunk::inuse(p) && chunk_top >= top {
                let tp = p.cast::<TreeChunk>();
                debug_assert!(Segment::holds(sp, sp.cast()));
                if p == self.dv {
                    self.dv = ptr::null_mut();
                    self.dvsize = 0;
                } else {
                    self.unlink_large_chunk(tp);
                }
                if syscall_free(base, size) {
                    released += size;
                    self.footprint -= size;
                    // unlink our obsolete record
                    sp = pred;
                    (*sp).next = next;
                } else {
                    // back out if we can't unmap
                    self.insert_large_chunk(tp, psize);
                }
            }
            pred = sp;
            sp = next;
        }
        self.release_checks = if nsegs > MAX_RELEASE_CHECK_RATE {
            nsegs
        } else {
            MAX_RELEASE_CHECK_RATE
        };
        released
    }

    // Sanity checks

    #[cfg(debug_assertions)]
    unsafe fn check_any_chunk(&self, p: *mut Chunk) {
        debug_assert!(
            Self::is_aligned(Chunk::to_mem(p) as usize) || (*p).head == Chunk::FENCEPOST_HEAD
        );
        debug_assert!(p.cast() >= self.least_addr);
    }

    #[cfg(debug_assertions)]
    unsafe fn check_top_chunk(&self, p: *mut Chunk) {
        let sp = self.segment_holding(p.cast());
        let sz = (*p).head & !INUSE;
        debug_assert!(!sp.is_null());
        debug_assert!(
            Self::is_aligned(Chunk::to_mem(p) as usize) || (*p).head == Chunk::FENCEPOST_HEAD
        );
        debug_assert!(p.cast() >= self.least_addr);
        debug_assert_eq!(sz, self.topsize);
        debug_assert!(sz > 0);
        debug_assert_eq!(
            sz,
            (*sp).base as usize + (*sp).size - p as usize - Self::top_foot_size()
        );
        debug_assert!(Chunk::pinuse(p));
        debug_assert!(!Chunk::pinuse(Chunk::plus_offset(p, sz)));
    }

    #[cfg(debug_assertions)]
    unsafe fn check_malloced_chunk(&self, mem: *mut u8, s: usize) {
        if mem.is_null() {
            return;
        }
        let p = Chunk::from_mem(mem);
        let sz = (*p).head & !INUSE;
        self.check_inuse_chunk(p);
        debug_assert_eq!(align_up(sz, Self::MALLOC_ALIGNMENT), sz);
        debug_assert!(sz >= Self::MIN_CHUNK_SIZE);
        debug_assert!(sz >= s);
        debug_assert!(Chunk::mmapped(p) || sz < (s + Self::MIN_CHUNK_SIZE));
    }

    #[cfg(debug_assertions)]
    unsafe fn check_inuse_chunk(&self, p: *mut Chunk) {
        self.check_any_chunk(p);
        debug_assert!(Chunk::inuse(p));
        debug_assert!(Chunk::pinuse(Chunk::next(p)));
        debug_assert!(Chunk::mmapped(p) || Chunk::pinuse(p) || Chunk::next(Chunk::prev(p)) == p);
        if Chunk::mmapped(p) {
            self.check_mmapped_chunk(p);
        }
    }

    #[cfg(debug_assertions)]
    unsafe fn check_mmapped_chunk(&self, p: *mut Chunk) {
        let sz = Chunk::size(p);
        let len = sz + (*p).prev_foot + Self::mmap_foot_pad();
        debug_assert!(Chunk::mmapped(p));
        debug_assert!(
            Self::is_aligned(Chunk::to_mem(p) as usize) || (*p).head == Chunk::FENCEPOST_HEAD
        );
        debug_assert!(p.cast::<u8>() >= self.least_addr);
        debug_assert!(!Self::is_small(sz));
        debug_assert_eq!(align_up(len, PAGE_SIZE), len);
        debug_assert_eq!((*Chunk::plus_offset(p, sz)).head, Chunk::FENCEPOST_HEAD);
        debug_assert_eq!(
            (*Chunk::plus_offset(p, sz + mem::size_of::<usize>())).head,
            0
        );
    }

    #[cfg(debug_assertions)]
    unsafe fn check_free_chunk(&self, p: *mut Chunk) {
        let sz = Chunk::size(p);
        let next = Chunk::plus_offset(p, sz);
        self.check_any_chunk(p);
        debug_assert!(!Chunk::inuse(p));
        debug_assert!(!Chunk::pinuse(Chunk::next(p)));
        debug_assert!(!Chunk::mmapped(p));
        if p != self.dv && p != self.top {
            if sz >= Self::MIN_CHUNK_SIZE {
                debug_assert_eq!(align_up(sz, Self::MALLOC_ALIGNMENT), sz);
                debug_assert!(Self::is_aligned(Chunk::to_mem(p) as usize));
                debug_assert_eq!((*next).prev_foot, sz);
                debug_assert!(Chunk::pinuse(p));
                debug_assert!(next == self.top || Chunk::inuse(next));
                debug_assert_eq!((*(*p).next).prev, p);
                debug_assert_eq!((*(*p).prev).next, p);
            } else {
                debug_assert_eq!(sz, mem::size_of::<usize>());
            }
        }
    }

    #[cfg(debug_assertions)]
    unsafe fn check_malloc_state(&mut self) {
        for i in 0..NSMALLBINS {
            self.check_smallbin(u32::try_from(i).unwrap());
        }
        for i in 0..NTREEBINS {
            self.check_treebin(u32::try_from(i).unwrap());
        }
        if self.dvsize != 0 {
            self.check_any_chunk(self.dv);
            debug_assert_eq!(self.dvsize, Chunk::size(self.dv));
            debug_assert!(self.dvsize >= Self::MIN_CHUNK_SIZE);
            let dv = self.dv;
            debug_assert!(!self.bin_find(dv));
        }
        if !self.top.is_null() {
            self.check_top_chunk(self.top);
            debug_assert!(self.topsize > 0);
            let top = self.top;
            debug_assert!(!self.bin_find(top));
        }
    }

    #[cfg(debug_assertions)]
    unsafe fn check_smallbin(&mut self, idx: u32) {
        if !cfg!(debug_assertions) {
            return;
        }
        let b = self.smallbin_at(idx);
        let mut p = (*b).next;
        let empty = self.smallmap & (1 << idx) == 0;
        if p == b {
            debug_assert!(empty);
        }
        if !empty {
            while p != b {
                let size = Chunk::size(p);
                self.check_free_chunk(p);
                debug_assert_eq!(Self::small_index(size), idx);
                debug_assert!((*p).next == b || Chunk::size((*p).next) == Chunk::size(p));
                let q = Chunk::next(p);
                if (*q).head != Chunk::FENCEPOST_HEAD {
                    self.check_inuse_chunk(q);
                }
                p = (*p).next;
            }
        }
    }

    #[cfg(debug_assertions)]
    unsafe fn check_treebin(&mut self, idx: u32) {
        let tb = self.treebin_at(idx);
        let t = *tb;
        let empty = self.treemap & (1 << idx) == 0;
        if t.is_null() {
            debug_assert!(empty);
        }
        if !empty {
            self.check_tree(t);
        }
    }

    #[cfg(debug_assertions)]
    unsafe fn check_tree(&mut self, t: *mut TreeChunk) {
        let tc = TreeChunk::chunk(t);
        let tindex = (*t).index;
        let tsize = Chunk::size(tc);
        let idx = Self::compute_tree_index(tsize);
        debug_assert_eq!(tindex, idx);
        debug_assert!(tsize >= Self::MIN_LARGE_SIZE);
        debug_assert!(tsize >= Self::min_size_for_tree_index(idx));
        debug_assert!(
            idx == u32::try_from(NTREEBINS).unwrap() - 1
                || tsize < Self::min_size_for_tree_index(idx + 1)
        );

        let mut u = t;
        let mut head = ptr::null_mut::<TreeChunk>();
        loop {
            let uc = TreeChunk::chunk(u);
            self.check_any_chunk(uc);
            debug_assert_eq!((*u).index, tindex);
            debug_assert_eq!(Chunk::size(uc), tsize);
            debug_assert!(!Chunk::inuse(uc));
            debug_assert!(!Chunk::pinuse(Chunk::next(uc)));
            debug_assert_eq!((*(*uc).next).prev, uc);
            debug_assert_eq!((*(*uc).prev).next, uc);
            let left = (*u).child[0];
            let right = (*u).child[1];
            if (*u).parent.is_null() {
                debug_assert!(left.is_null());
                debug_assert!(right.is_null());
            } else {
                debug_assert!(head.is_null());
                head = u;
                debug_assert!((*u).parent != u);
                debug_assert!(
                    (*(*u).parent).child[0] == u
                        || (*(*u).parent).child[1] == u
                        || *((*u).parent.cast::<*mut TreeChunk>()) == u
                );
                if !left.is_null() {
                    debug_assert_eq!((*left).parent, u);
                    debug_assert!(left != u);
                    self.check_tree(left);
                }
                if !right.is_null() {
                    debug_assert_eq!((*right).parent, u);
                    debug_assert!(right != u);
                    self.check_tree(right);
                }
                if !left.is_null() && !right.is_null() {
                    debug_assert!(
                        Chunk::size(TreeChunk::chunk(left)) < Chunk::size(TreeChunk::chunk(right))
                    );
                }
            }

            u = TreeChunk::prev(u);
            if u == t {
                break;
            }
        }
        debug_assert!(!head.is_null());
    }

    #[cfg(debug_assertions)]
    fn min_size_for_tree_index(idx: u32) -> usize {
        let idx = idx as usize;
        (1 << ((idx >> 1) + TREEBIN_SHIFT)) | ((idx & 1) << ((idx >> 1) + TREEBIN_SHIFT - 1))
    }

    #[cfg(debug_assertions)]
    unsafe fn bin_find(&mut self, chunk: *mut Chunk) -> bool {
        let size = Chunk::size(chunk);
        if Self::is_small(size) {
            let sidx = Self::small_index(size);
            let b = self.smallbin_at(sidx);
            if !self.smallmap_is_marked(sidx) {
                return false;
            }
            let mut p = b;
            loop {
                if p == chunk {
                    return true;
                }
                p = (*p).prev;
                if p == b {
                    return false;
                }
            }
        } else {
            let tidx = Self::compute_tree_index(size);
            if !self.treemap_is_marked(tidx) {
                return false;
            }
            let mut t = *self.treebin_at(tidx);
            let mut sizebits = size << leftshift_for_tree_index(tidx);
            while !t.is_null() && Chunk::size(TreeChunk::chunk(t)) != size {
                t = (*t).child[(sizebits >> (mem::size_of::<usize>() * 8 - 1)) & 1];
                sizebits <<= 1;
            }
            if t.is_null() {
                return false;
            }
            let mut u = t;
            let chunk = chunk.cast::<TreeChunk>();
            loop {
                if u == chunk {
                    return true;
                }
                u = TreeChunk::prev(u);
                if u == t {
                    return false;
                }
            }
        }
    }
}

const PINUSE: usize = 1 << 0;
const CINUSE: usize = 1 << 1;
const FLAG4: usize = 1 << 2;
const INUSE: usize = PINUSE | CINUSE;
const FLAG_BITS: usize = PINUSE | CINUSE | FLAG4;

impl Chunk {
    const FENCEPOST_HEAD: usize = INUSE | mem::size_of::<usize>();
    const MEM_OFFSET: usize = 2 * (mem::size_of::<usize>());

    #[inline]
    unsafe fn size(me: *mut Chunk) -> usize {
        (*me).head & !FLAG_BITS
    }

    #[expect(clippy::cast_ptr_alignment)]
    #[cfg(debug_assertions)]
    unsafe fn next(me: *mut Chunk) -> *mut Chunk {
        (me.cast::<u8>())
            .add((*me).head & !FLAG_BITS)
            .cast::<Chunk>()
    }

    #[expect(clippy::cast_ptr_alignment)]
    #[cfg(debug_assertions)]
    unsafe fn prev(me: *mut Chunk) -> *mut Chunk {
        (me.cast::<u8>()).sub((*me).prev_foot).cast::<Chunk>()
    }

    #[inline]
    unsafe fn cinuse(me: *mut Chunk) -> bool {
        (*me).head & CINUSE != 0
    }

    #[inline]
    unsafe fn pinuse(me: *mut Chunk) -> bool {
        (*me).head & PINUSE != 0
    }

    #[inline]
    unsafe fn clear_pinuse(me: *mut Chunk) {
        (*me).head &= !PINUSE;
    }

    #[inline]
    unsafe fn inuse(me: *mut Chunk) -> bool {
        (*me).head & INUSE != PINUSE
    }

    #[inline]
    unsafe fn mmapped(me: *mut Chunk) -> bool {
        (*me).head & INUSE == 0
    }

    unsafe fn set_inuse(me: *mut Chunk, size: usize) {
        (*me).head = ((*me).head & PINUSE) | size | CINUSE;
        let next = Chunk::plus_offset(me, size);
        (*next).head |= PINUSE;
    }

    unsafe fn set_inuse_and_pinuse(me: *mut Chunk, size: usize) {
        (*me).head = PINUSE | size | CINUSE;
        let next = Chunk::plus_offset(me, size);
        (*next).head |= PINUSE;
    }

    unsafe fn set_size_and_pinuse_of_inuse_chunk(me: *mut Chunk, size: usize) {
        (*me).head = size | PINUSE | CINUSE;
    }

    unsafe fn set_size_and_pinuse_of_free_chunk(me: *mut Chunk, size: usize) {
        (*me).head = size | PINUSE;
        Chunk::set_foot(me, size);
    }

    unsafe fn set_free_with_pinuse(p: *mut Chunk, size: usize, n: *mut Chunk) {
        Chunk::clear_pinuse(n);
        Chunk::set_size_and_pinuse_of_free_chunk(p, size);
    }

    unsafe fn set_foot(me: *mut Chunk, size: usize) {
        let next = Chunk::plus_offset(me, size);
        (*next).prev_foot = size;
    }

    #[expect(clippy::cast_ptr_alignment)]
    unsafe fn plus_offset(me: *mut Chunk, offset: usize) -> *mut Chunk {
        (me.cast::<u8>()).add(offset).cast::<Chunk>()
    }

    #[expect(clippy::cast_ptr_alignment)]
    unsafe fn minus_offset(me: *mut Chunk, offset: usize) -> *mut Chunk {
        (me.cast::<u8>()).sub(offset).cast::<Chunk>()
    }

    #[inline]
    unsafe fn to_mem(me: *mut Chunk) -> *mut u8 {
        (me.cast::<u8>()).add(Chunk::MEM_OFFSET)
    }

    #[expect(clippy::cast_possible_wrap, clippy::cast_ptr_alignment)]
    unsafe fn from_mem(mem: *mut u8) -> *mut Chunk {
        mem.offset(-2 * (mem::size_of::<usize>() as isize))
            .cast::<Chunk>()
    }
}

impl TreeChunk {
    unsafe fn leftmost_child(me: *mut TreeChunk) -> *mut TreeChunk {
        let left = (*me).child[0];
        if left.is_null() {
            (*me).child[1]
        } else {
            left
        }
    }

    unsafe fn chunk(me: *mut TreeChunk) -> *mut Chunk {
        &raw mut (*me).chunk
    }

    unsafe fn next(me: *mut TreeChunk) -> *mut TreeChunk {
        (*TreeChunk::chunk(me)).next.cast::<TreeChunk>()
    }

    unsafe fn prev(me: *mut TreeChunk) -> *mut TreeChunk {
        (*TreeChunk::chunk(me)).prev.cast::<TreeChunk>()
    }
}

impl Segment {
    unsafe fn sys_flags(seg: *mut Segment) -> u32 {
        (*seg).flags >> 1
    }

    unsafe fn holds(seg: *mut Segment, addr: *mut u8) -> bool {
        (*seg).base <= addr && addr < Segment::top(seg)
    }

    unsafe fn top(seg: *mut Segment) -> *mut u8 {
        (*seg).base.add((*seg).size)
    }
}

fn syscall_alloc(size: usize) -> (*mut u8, usize, u32) {
    let addr = unsafe { syscall!(MMAP, 0, size, 2 | 1, 0x0020 | 0x0002, -1isize, 0) };
    if is_syscall_error(addr) {
        (ptr::null_mut(), 0, 0)
    } else {
        (addr as *mut u8, size, 0)
    }
}

fn syscall_remap(ptr: *mut u8, oldsize: usize, newsize: usize, can_move: bool) -> *mut u8 {
    let flags = i32::from(can_move);
    let ptr = unsafe { syscall!(MREMAP, ptr, oldsize, newsize, flags) };
    if is_syscall_error(ptr) {
        ptr::null_mut()
    } else {
        ptr as *mut u8
    }
}

fn syscall_free_part(ptr: *mut u8, oldsize: usize, newsize: usize) -> bool {
    unsafe {
        let remap_ptr = syscall!(MREMAP, ptr, oldsize, newsize, 0);
        if is_syscall_error(remap_ptr) {
            syscall!(MUNMAP, ptr.add(newsize), oldsize - newsize) == 0
        } else {
            true
        }
    }
}

fn syscall_free(ptr: *mut u8, size: usize) -> bool {
    unsafe { syscall!(MUNMAP, ptr, size) == 0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unix::random::Prng;

    // Prime the allocator with some allocations such that there will be free
    // chunks in the treemap
    unsafe fn setup_treemap(a: &mut Dlmalloc) {
        let large_request_size = NSMALLBINS * (1 << SMALLBIN_SHIFT);
        assert!(!Dlmalloc::is_small(large_request_size));
        let large_request1 = a.inner_malloc(large_request_size);
        assert_ne!(large_request1, ptr::null_mut());
        let large_request2 = a.inner_malloc(large_request_size);
        assert_ne!(large_request2, ptr::null_mut());
        a.free(large_request1);
        assert_ne!(a.treemap, 0);
    }

    #[test]
    // Test allocating, with a non-empty treemap, a specific size that used to
    // trigger an integer overflow bug
    fn treemap_alloc_overflow_minimal() {
        let mut a = Dlmalloc::new();
        unsafe {
            setup_treemap(&mut a);
            let min_idx31_size = (0xc000 << TREEBIN_SHIFT) - Dlmalloc::CHUNK_OVERHEAD + 1;
            assert_ne!(a.inner_malloc(min_idx31_size), ptr::null_mut());
        }
    }

    #[test]
    // Test allocating the maximum request size with a non-empty treemap
    fn treemap_alloc_max() {
        let mut a = Dlmalloc::new();
        unsafe {
            setup_treemap(&mut a);
            let max_request_size = Dlmalloc::MAX_REQUEST - 1;
            assert_eq!(a.inner_malloc(max_request_size), ptr::null_mut());
        }
    }

    #[test]
    fn smoke() {
        let mut a = Dlmalloc::new();
        let aligns = [1, 2, 4, 8, 16];
        for i in 0..22 {
            let align = aligns[i % aligns.len()];
            unsafe {
                let tiny_ptr = a.malloc(1, 1);
                assert!(!tiny_ptr.is_null());
                *tiny_ptr = 9;
                assert_eq!(*tiny_ptr, 9);

                let small_ptr = a.malloc(16, 2);
                assert!(!small_ptr.is_null());
                *small_ptr = 10;
                assert_eq!(*small_ptr, 10);
                let large_ptr = a.malloc(2_u64.pow(i as u32) as usize, align);
                a.free(large_ptr);
                a.free(small_ptr);
                a.free(tiny_ptr);
            }
        }
    }

    #[test]
    fn realloc_small_align() {
        let mut a = Dlmalloc::new();
        let aligns = [1, 2, 4, 8, 16];
        for i in 0..22 {
            let align = aligns[i % aligns.len()];
            unsafe {
                let sz = 2_u64.pow(i as u32) as usize;
                let ptr1 = a.malloc(2_u64.pow(i as u32) as usize, align);
                ptr1.write_bytes(1, sz);
                let new_size = 2_u64.pow(i as u32 + 1) as usize;
                let ptr2 = a.realloc(ptr1, sz, align, new_size);
                ptr2.add(sz).write_bytes(2, sz);
                let mut sum = 0usize;
                for j in 0..new_size {
                    sum += ptr2.add(j).read() as usize;
                }
                // Should have written 1 to the first half, and 2 to the second half
                assert_eq!(sz * 3, sum);
                a.free(ptr2);
            }
        }
    }

    #[test]
    fn realloc_big_align() {
        let mut a = Dlmalloc::new();
        let aligns = [32, 64, 128, 256, 512];
        for i in 9..22 {
            let align = aligns[i % aligns.len()];
            unsafe {
                let sz = 2_u64.pow(i as u32) as usize;
                let ptr1 = a.malloc(2_u64.pow(i as u32) as usize, align);
                ptr1.write_bytes(1, sz);
                let new_size = 2_u64.pow(i as u32 + 1) as usize;
                let ptr2 = a.realloc(ptr1, sz, align, new_size);
                ptr2.add(sz).write_bytes(2, sz);
                let mut sum = 0usize;
                for j in 0..new_size {
                    sum += ptr2.add(j).read() as usize;
                }
                // Should have written 1 to the first half, and 2 to the second half
                assert_eq!(sz * 3, sum);
                a.free(ptr2);
            }
        }
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn stress() {
        use alloc::vec::Vec;
        let mut a = Dlmalloc::new();
        let mut rng = Prng::new_time_seeded();
        let mut ptrs = Vec::new();
        let max = 1_000;
        unsafe {
            for _ in 0..max {
                let free = ptrs.len() > 0
                    && ((ptrs.len() < 10_000 && weighted_bool(&mut rng, 3))
                        || random_bool(&mut rng));
                if free {
                    let idx = random_gen_range(&mut rng, 0, ptrs.len());
                    let (ptr, _size, _align) = ptrs.swap_remove(idx);
                    a.free(ptr);
                    continue;
                }

                if ptrs.len() > 0 && weighted_bool(&mut rng, 100) {
                    let idx = random_gen_range(&mut rng, 0, ptrs.len());
                    let (ptr, size, align) = ptrs.swap_remove(idx);
                    let new_size = if random_bool(&mut rng) {
                        random_gen_range(&mut rng, size, size * 2)
                    } else if size > 10 {
                        random_gen_range(&mut rng, size / 2, size)
                    } else {
                        continue;
                    };
                    let mut tmp = Vec::new();
                    for i in 0..cmp::min(size, new_size) {
                        tmp.push(*ptr.offset(i as isize));
                    }
                    let ptr = a.realloc(ptr, size, align, new_size);
                    assert!(!ptr.is_null());
                    for (i, byte) in tmp.iter().enumerate() {
                        assert_eq!(*byte, *ptr.offset(i as isize));
                    }
                    ptrs.push((ptr, new_size, align));
                }

                let size = if random_bool(&mut rng) {
                    random_gen_range(&mut rng, 1, 128)
                } else {
                    random_gen_range(&mut rng, 1, 128 * 1024)
                };
                let align = if weighted_bool(&mut rng, 10) {
                    1 << random_gen_range(&mut rng, 3, 8)
                } else {
                    8
                };

                let zero = weighted_bool(&mut rng, 50);
                let ptr = if zero {
                    a.calloc(size, align)
                } else {
                    a.malloc(size, align)
                };
                for i in 0..size {
                    if zero {
                        assert_eq!(*ptr.offset(i as isize), 0);
                    }
                    *ptr.offset(i as isize) = 0xce;
                }
                ptrs.push((ptr, size, align));
            }
        }
    }

    fn weighted_bool(prng: &mut Prng, weight: u64) -> bool {
        let max = u64::MAX / weight;
        let next = prng.next_u64();
        next < max
    }

    fn random_bool(prng: &mut Prng) -> bool {
        prng.next_u64() & 1 == 0
    }

    fn random_gen_range(prng: &mut Prng, low: usize, high: usize) -> usize {
        let lf = low as f64;
        let hf = high as f64;
        let res = if low == 0 {
            let transform = hf / u64::MAX as f64;
            (prng.next_u64() as f64 * transform) as usize
        } else {
            let space = hf - lf;
            let transform = space / u64::MAX as f64;
            (prng.next_u64() as f64 * transform + space) as usize
        };
        res
    }
}
