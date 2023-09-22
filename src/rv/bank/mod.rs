mod error;

use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use crate::rv::bank::error::EntryError;

const MAGIC_DECOMPRESSED: i32 = 0x00000000;
const MAGIC_COMPRESSED: i32 = 0x43707273;
const MAGIC_ENCRYPTED: i32 = 0x456e6372;
const MAGIC_VERSION: i32 = 0x56657273;

enum EntryMime {
    Decompressed,
    Compressed,
    Encrypted
}

trait Directory {

    //entries: HashMap<String, BankEntry>
    fn get_entries(&self) -> &Vec<BankEntry>;

    fn get_entry(&self, name: &str) -> Result<&BankEntry, dyn Error> {
        match self.get_entries().iter().find(|e| e.name == name) {
            Some(entry) => Ok(entry),
            None => Err(Box::new(EntryError::EntryNotFound)),
        }
    }
}

trait File {
    fn get_mime(&self) -> &EntryMime;
    fn get_contents(&self) -> Arc<Vec<u8>>;
}

pub enum EntryContent {
    Directory(dyn Directory),
    File(dyn File)
}

pub struct BankEntry {
    name: String,
    content: EntryContent,
}


struct BankArchive {
    file_name: String,
    bank_properties: HashMap<String, String>,
    entries: HashMap<String, Arc<BankEntry>>
}