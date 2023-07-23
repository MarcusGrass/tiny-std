use crate::platform::{DirType, InoT, OffT, NULL_BYTE};

#[derive(Copy, Clone)]
pub struct Dirent {
    pub d_ino: InoT,
    pub d_off: OffT,
    pub d_reclen: u16,
    pub d_type: DirType,
    pub d_name: [u8; 256],
    // <- One padding byte here
}

impl Dirent {
    const LEN_OFFSET: usize = core::mem::size_of::<InoT>() + core::mem::size_of::<OffT>();
    const HEADER_SIZE: usize = Self::LEN_OFFSET + 2;
    const INOT_SIZE: usize = core::mem::size_of::<InoT>();
    const OFFT_SIZE: usize = core::mem::size_of::<OffT>();
    const NAME_START: usize = Self::HEADER_SIZE + 1;

    /// Try to parse a Dirent from the memory inside of a buffer filled by the `GETDENTS64` syscall
    /// # Safety
    /// Essentially only safe if used specifically on a buffer filled by `get_dents` starting with
    /// a valid offset which would be `0 + n1 + ... + nm` where n is the `d_reclen`
    /// of the `m`th already parsed `Dirent` from that same buffer.
    #[inline]
    #[must_use]
    #[allow(clippy::transmute_ptr_to_ptr)]
    pub unsafe fn try_from_bytes(buf: &[u8]) -> Option<Self> {
        let header = buf.get(0..Self::HEADER_SIZE)?;
        // Could get_unchecked
        let len = {
            // Safety: We just checked it's in range, transmute the two bytes into a single ne-u16, get it
            let len_buf = header.get_unchecked(Self::LEN_OFFSET..Self::HEADER_SIZE);
            *core::mem::transmute::<&[u8], &[u16]>(len_buf).get_unchecked(0)
        };
        let d_type = *buf.get_unchecked(Self::HEADER_SIZE);
        let mut name = [NULL_BYTE; 256];
        let bytes = buf.get_unchecked(Self::NAME_START..);
        for (ind, byte) in bytes.iter().enumerate() {
            let byte = *byte;
            if byte == NULL_BYTE {
                break;
            }
            name[ind] = byte;
        }
        Some(Self {
            d_ino: InoT::from_ne_bytes(
                buf.get_unchecked(0..Self::INOT_SIZE)
                    .try_into()
                    .unwrap_unchecked(),
            ),
            d_off: OffT::from_ne_bytes(
                buf.get_unchecked(Self::INOT_SIZE..Self::INOT_SIZE + Self::OFFT_SIZE)
                    .try_into()
                    .unwrap_unchecked(),
            ),
            d_reclen: len,
            d_name: name,
            d_type: DirType(d_type),
        })
    }
}
