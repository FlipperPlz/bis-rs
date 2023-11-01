
pub mod path;
pub mod io;

use std::collections::HashMap;
use std::io::{Read, Seek};
use crate::bank::io::{BankSkimOptions, EntryError, EntryMetadataError};
use crate::rv::io::PboReader;
use crate::magic_enum;

magic_enum! {
    i32,
    EntryMime,
    EntryMetadataError,
    EntryMimeNotSupported {
        Decompressed = 0x00000000,
        Compressed   = 0x43707273,
        Encrypted    = 0x456e6372,
        Version      = 0x56657273
    }
}


#[derive(Clone, Debug)]
pub struct PboFileSkim<R: Read + Seek> {
    pub(crate) reader:        PboReader<R>,
    pub(crate) entries:       HashMap<BankSkimEntry, u64>,
    pub(crate) options:       BankSkimOptions,
    pub(crate) properties:    HashMap<String, String>
}

impl<R: Read + Seek> PboFileSkim<R> {
    pub fn get_entry(&self, entry_name: &str) -> Option<&BankSkimEntry> {
        self.entries.keys().find(|&entry| {
            entry.filename.eq_ignore_ascii_case(entry_name)
        })
    }

    pub fn read_entry(&mut self, entry: &BankSkimEntry) -> Result<Vec<u8>, EntryError> {
        self.reader.read_entry_data(entry, self.entries.get(entry).unwrap())
    }
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct BankSkimEntry {
    pub(crate) filename:      String,
    pub(crate) mime: EntryMime,
    pub(crate) size_unpacked: u32,
    pub(crate) start_offset:  u64,
    pub(crate) timestamp:     u32,
    pub(crate) size_packed:   u32,
}




