//! Symbols that rustc assumes are present and will emit causing linking breakage if not
//! provided.
//! Taken almost straight from [compiler-builtins](https://github.com/rust-lang/compiler-builtins)
//! while stripping some platform specifics irrelevant for the targets of `x86_64` and `aarch64`,
//! as well as some `intrinsics`-calls (`likely`) which cannot be used on stable.
//!
//! The code is licensed under MIT and the license can be found in the above repo, also reproduced
//! below:
//
// ==============================================================================
// compiler-builtins License
// ==============================================================================
//
// The compiler-builtins crate is dual licensed under both the University of
// Illinois "BSD-Like" license and the MIT license.  As a user of this code you may
// choose to use it under either license.  As a contributor, you agree to allow
// your code to be used under both.
//
// Full text of the relevant licenses is included below.
//
// ==============================================================================
//
// University of Illinois/NCSA
// Open Source License
//
// Copyright (c) 2009-2016 by the contributors listed in CREDITS.TXT
//
// All rights reserved.
//
// Developed by:
//
// LLVM Team
//
// University of Illinois at Urbana-Champaign
//
// http://llvm.org
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of
// this software and associated documentation files (the "Software"), to deal with
// the Software without restriction, including without limitation the rights to
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
// of the Software, and to permit persons to whom the Software is furnished to do
// so, subject to the following conditions:
//
// * Redistributions of source code must retain the above copyright notice,
// this list of conditions and the following disclaimers.
//
// * Redistributions in binary form must reproduce the above copyright notice,
// this list of conditions and the following disclaimers in the
// documentation and/or other materials provided with the distribution.
//
// * Neither the names of the LLVM Team, University of Illinois at
// Urbana-Champaign, nor the names of its contributors may be used to
// endorse or promote products derived from this Software without specific
// prior written permission.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
// FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.  IN NO EVENT SHALL THE
// CONTRIBUTORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS WITH THE
// SOFTWARE.
//
// ==============================================================================
//
// Copyright (c) 2009-2015 by the contributors listed in CREDITS.TXT
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.
//
// ==============================================================================
// Copyrights and Licenses for Third Party Software Distributed with LLVM:
// ==============================================================================
// The LLVM software contains code written by third parties.  Such software will
// have its own individual LICENSE.TXT file in the directory in which it appears.
// This file will describe the copyrights, license, and restrictions which apply
// to that code.
//
// The disclaimer of warranty in the University of Illinois Open Source License
// applies to all code in the LLVM Distribution, and nothing in any of the
// other licenses gives permission to use the names of the LLVM Team or the
// University of Illinois to endorse or promote products derived from this
// Software.
//

const WORD_SIZE: usize = core::mem::size_of::<usize>();
const WORD_MASK: usize = WORD_SIZE - 1;

// If the number of bytes involved exceed this threshold we will opt in word-wise copy.
// The value here selected is max(2 * WORD_SIZE, 16):
// * We need at least 2 * WORD_SIZE bytes to guarantee that at least 1 word will be copied through
//   word-wise copy.
// * The word-wise copy logic needs to perform some checks so it has some small overhead.
//   ensures that even on 32-bit platforms we have copied at least 8 bytes through
//   word-wise copy so the saving of word-wise copy outweights the fixed overhead.
const WORD_COPY_THRESHOLD: usize = if 2 * WORD_SIZE > 16 {
    2 * WORD_SIZE
} else {
    16
};

#[no_mangle]
#[inline(always)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    copy_forward(dest, src, n);
    dest
}

#[no_mangle]
#[inline(always)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn memmove(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let delta = (dest as usize).wrapping_sub(src as usize);
    if delta >= n {
        // We can copy forwards because either dest is far enough ahead of src,
        // or src is ahead of dest (and delta overflowed).
        copy_forward(dest, src, n);
    } else {
        copy_backward(dest, src, n);
    }
    dest
}

#[no_mangle]
#[inline(always)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    compare_bytes(s1, s2, n)
}

#[no_mangle]
#[inline(always)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn bcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    memcmp(s1, s2, n)
}

#[no_mangle]
#[inline(always)]
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::missing_safety_doc
)]
pub unsafe extern "C" fn memset(s: *mut u8, c: core::ffi::c_int, n: usize) -> *mut u8 {
    set_bytes(s, c as u8, n);
    s
}

#[inline(always)]
unsafe fn compare_bytes(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    let mut i = 0;
    while i < n {
        let a = *s1.add(i);
        let b = *s2.add(i);
        if a != b {
            return i32::from(a) - i32::from(b);
        }
        i += 1;
    }
    0
}

#[inline(always)]
unsafe fn copy_forward(mut dest: *mut u8, mut src: *const u8, mut n: usize) {
    #[inline(always)]
    unsafe fn copy_forward_bytes(mut dest: *mut u8, mut src: *const u8, n: usize) {
        let dest_end = dest.add(n);
        while dest < dest_end {
            *dest = *src;
            dest = dest.add(1);
            src = src.add(1);
        }
    }

    #[inline(always)]
    #[allow(clippy::cast_ptr_alignment)]
    unsafe fn copy_forward_aligned_words(dest: *mut u8, src: *const u8, n: usize) {
        let mut dest_usize = dest.cast::<usize>();
        let mut src_usize = src as *mut usize;
        let dest_end = dest.add(n).cast::<usize>();

        while dest_usize < dest_end {
            *dest_usize = *src_usize;
            dest_usize = dest_usize.add(1);
            src_usize = src_usize.add(1);
        }
    }

    /// Both `x86_64` and aarch support mem-unaligned
    #[inline(always)]
    #[allow(clippy::cast_ptr_alignment)]
    unsafe fn copy_forward_misaligned_words(dest: *mut u8, src: *const u8, n: usize) {
        let mut dest_usize = dest.cast::<usize>();
        let mut src_usize = src as *mut usize;
        let dest_end = dest.add(n).cast::<usize>();

        while dest_usize < dest_end {
            *dest_usize = read_usize_unaligned(src_usize);
            dest_usize = dest_usize.add(1);
            src_usize = src_usize.add(1);
        }
    }

    if n >= WORD_COPY_THRESHOLD {
        // Align dest
        // Because of n >= 2 * WORD_SIZE, dst_misalignment < n
        let dest_misalignment = (dest as usize).wrapping_neg() & WORD_MASK;
        copy_forward_bytes(dest, src, dest_misalignment);
        dest = dest.add(dest_misalignment);
        src = src.add(dest_misalignment);
        n -= dest_misalignment;

        let n_words = n & !WORD_MASK;
        let src_misalignment = src as usize & WORD_MASK;
        if src_misalignment == 0 {
            copy_forward_aligned_words(dest, src, n_words);
        } else {
            copy_forward_misaligned_words(dest, src, n_words);
        }
        dest = dest.add(n_words);
        src = src.add(n_words);
        n -= n_words;
    }
    copy_forward_bytes(dest, src, n);
}

unsafe fn read_usize_unaligned(x: *const usize) -> usize {
    // Do not use `core::ptr::read_unaligned` here, since it calls `copy_nonoverlapping` which
    // is translated to memcpy in LLVM.
    let x_read = x.cast::<[u8; 8]>().read();
    core::mem::transmute(x_read)
}

#[inline(always)]
unsafe fn copy_backward(dest: *mut u8, src: *const u8, mut n: usize) {
    // The following backward copy helper functions uses the pointers past the end
    // as their inputs instead of pointers to the start!
    #[inline(always)]
    unsafe fn copy_backward_bytes(mut dest: *mut u8, mut src: *const u8, n: usize) {
        let dest_start = dest.sub(n);
        while dest_start < dest {
            dest = dest.sub(1);
            src = src.sub(1);
            *dest = *src;
        }
    }

    #[inline(always)]
    #[allow(clippy::cast_ptr_alignment)]
    unsafe fn copy_backward_aligned_words(dest: *mut u8, src: *const u8, n: usize) {
        let mut dest_usize = dest.cast::<usize>();
        let mut src_usize = src as *mut usize;
        let dest_start = dest.sub(n).cast::<usize>();

        while dest_start < dest_usize {
            dest_usize = dest_usize.sub(1);
            src_usize = src_usize.sub(1);
            *dest_usize = *src_usize;
        }
    }

    #[inline(always)]
    #[allow(clippy::cast_ptr_alignment)]
    unsafe fn copy_backward_misaligned_words(dest: *mut u8, src: *const u8, n: usize) {
        let mut dest_usize = dest.cast::<usize>();
        let mut src_usize = src as *mut usize;
        let dest_start = dest.sub(n).cast::<usize>();

        while dest_start < dest_usize {
            dest_usize = dest_usize.sub(1);
            src_usize = src_usize.sub(1);
            *dest_usize = read_usize_unaligned(src_usize);
        }
    }

    let mut dest = dest.add(n);
    let mut src = src.add(n);

    if n >= WORD_COPY_THRESHOLD {
        // Align dest
        // Because of n >= 2 * WORD_SIZE, dst_misalignment < n
        let dest_misalignment = dest as usize & WORD_MASK;
        copy_backward_bytes(dest, src, dest_misalignment);
        dest = dest.sub(dest_misalignment);
        src = src.sub(dest_misalignment);
        n -= dest_misalignment;

        let n_words = n & !WORD_MASK;
        let src_misalignment = src as usize & WORD_MASK;
        if src_misalignment == 0 {
            copy_backward_aligned_words(dest, src, n_words);
        } else {
            copy_backward_misaligned_words(dest, src, n_words);
        }
        dest = dest.sub(n_words);
        src = src.sub(n_words);
        n -= n_words;
    }
    copy_backward_bytes(dest, src, n);
}

#[inline(always)]
unsafe fn set_bytes(mut s: *mut u8, c: u8, mut n: usize) {
    #[inline(always)]
    unsafe fn set_bytes_bytes(mut s: *mut u8, c: u8, n: usize) {
        let end = s.add(n);
        while s < end {
            *s = c;
            s = s.add(1);
        }
    }

    #[inline(always)]
    #[allow(clippy::cast_ptr_alignment)]
    unsafe fn set_bytes_words(s: *mut u8, c: u8, n: usize) {
        let mut broadcast = c as usize;
        let mut bits = 8;
        while bits < WORD_SIZE * 8 {
            broadcast |= broadcast << bits;
            bits *= 2;
        }

        let mut s_usize = s.cast::<usize>();
        let end = s.add(n).cast::<usize>();

        while s_usize < end {
            *s_usize = broadcast;
            s_usize = s_usize.add(1);
        }
    }

    if n >= WORD_COPY_THRESHOLD {
        // Align s
        // Because of n >= 2 * WORD_SIZE, dst_misalignment < n
        let misalignment = (s as usize).wrapping_neg() & WORD_MASK;
        set_bytes_bytes(s, c, misalignment);
        s = s.add(misalignment);
        n -= misalignment;

        let n_words = n & !WORD_MASK;
        set_bytes_words(s, c, n_words);
        s = s.add(n_words);
        n -= n_words;
    }
    set_bytes_bytes(s, c, n);
}
