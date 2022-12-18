use crate::platform::NULL_BYTE;

pub mod strlen;
#[cfg(test)]
mod test;
pub mod unix_str;

/// Basic compare two null terminated strings
/// # Safety
/// Instant UB if these pointers are not null terminated
#[inline]
#[must_use]
pub unsafe fn null_term_ptr_cmp_up_to(a: *const u8, b: *const u8) -> usize {
    let mut it = 0;
    loop {
        let a_val = a.add(it).read();
        let b_val = b.add(it).read();
        if a_val != b_val || a_val == NULL_BYTE {
            // Not equal, or terminated
            return it;
        }
        // Equal continue
        it += 1;
    }
}
