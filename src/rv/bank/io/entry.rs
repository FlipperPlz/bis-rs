use crate::{magic_enum};
use crate::EntryMetadataError;

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct BankEntry {
    pub(crate) filename:      String,
    pub(crate) mime:          EntyMime,
    pub(crate) size_unpacked: u32,
    pub(crate) start_offset:  u64,
    pub(crate) timestamp:     u32,
    pub(crate) size_packed:   u32,
}

magic_enum! {
    i32,
    EntyMime,
    EntryMetadataError,
    EntryMimeNotSupported {
        Decompressed = 0x00000000,
        Compressed   = 0x43707273,
        Encrypted    = 0x456e6372,
        Version      = 0x56657273
    }
}

