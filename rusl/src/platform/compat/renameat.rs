transparent_bitflags! {
    pub struct RenameFlags: u32 {
        const RENAME_NOREPLACE = linux_rust_bindings::fs::RENAME_NOREPLACE as u32;
        const RENAME_EXCHANGE = linux_rust_bindings::fs::RENAME_EXCHANGE as u32;
        const RENAME_WHITEOUT = linux_rust_bindings::fs::RENAME_WHITEOUT as u32;
    }
}
