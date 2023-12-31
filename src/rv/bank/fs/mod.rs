mod error;

use std::collections::HashMap;
use std::fmt::{Debug};
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::Cursor;
use std::path::Path;
use crate::{BankSkimEntry, EntryMime, HEADER_PREFIX_MAGIC, PboFileSkim, PboReader};
use crate::fs::error::BankLoadError;
use crate::options::BankSkimOptions;


#[derive(Debug)]
struct BankFilesystem {
    banks:    Vec<BankFileMeta>,
}

#[derive(Debug)]
struct BankFileMeta {
    skim:              PboFileSkim<File>,
    prefix:            String,
    changed_prefix:    Option<String>,
    open_entries:      HashMap<CachedEntry, Cursor<Vec<u8>>>,
    deleted_entries:   Vec<String>,
}

#[derive(Debug, Hash)]
struct CachedEntry {
    cached_from:   Option<Box<BankSkimEntry>>,
    data_altered:  bool,
    name:          String,
    changed_name:  Option<String>,
    timestamp:     Option<CachedTimestamp>,
    offset:        Option<u32>,
    size:          Option<u32>,
    packed_size:   Option<u32>,
    mime:          Option<EntryMime>,
}

#[derive(Debug, Hash)]
enum CachedTimestamp {
    Generate,
    Custom(u32)
}

impl BankFileMeta {
    pub fn locate_file(&self, name: &str) -> Option<&CachedEntry> {
        self.open_entries.iter()
            .find(|(&ref entry, _)| entry.name.eq_ignore_ascii_case(name))
            .map(|(entry, _)| entry)
        // Look in skim if not found
    }

    // pub fn load_file(&self, skim_entry: &BankSkimEntry) -> {
    //     match self.skim.entries.get(skim_entry) {
    //         None => {}
    //         Some(&offset) => {}
    //     }
    // }
}

impl BankFilesystem {
    pub fn bank_for_prefix(&self, prefix: &str) -> Option<&BankFileMeta> {
        self.banks.iter().find(|&meta| {
            match &meta.changed_prefix {
                None => meta.prefix.eq_ignore_ascii_case(prefix),
                Some(it) => it.eq_ignore_ascii_case(prefix)
            }
        })
    }

    pub fn load_bank(&mut self, path: &Path, options: BankSkimOptions) -> Result<(), BankLoadError> {
        let file = File::open(path)?;
        let archive = PboReader::skim_archive(file, options)?;
        let prefix = match archive.properties.get(HEADER_PREFIX_MAGIC) {
            None => match path.file_stem() {
                None => Err(BankLoadError::FileNameUnknown),
                Some(it) => Ok(
                    it.to_str().ok_or(BankLoadError::FileNameUnknown)?.to_string()
                )
            }
            Some(it) => Ok(it.clone())
        }?;

        self.banks.push(BankFileMeta::new(prefix, archive));
        Ok(())
    }

}

impl BankFileMeta {
    fn new(prefix: String, skim: PboFileSkim<File>) -> Self {
        Self {
            skim,
            prefix,
            changed_prefix: None,
            open_entries: HashMap::new(),
            deleted_entries: vec![],
        }
    }

    fn unchanged(&self) -> bool {
        self.changed_prefix.is_none() &&
            self.open_entries.iter().all(|(entry, _)| entry.unchanged()) &&
            self.deleted_entries.is_empty()
    }
}

impl CachedEntry {
    fn unchanged(&self) -> bool {
        !self.data_altered &&
            self.timestamp.is_none() &&
            self.changed_name.is_none() &&
            self.packed_size.is_none() &&
            self.size.is_none() &&
            self.mime.is_none() &&
            self.offset.is_none()
    }
}


// impl FileSystem for BankFilesystem {
//
//     fn read_dir(&self, path: &str) -> VfsResult<Box<dyn Iterator<Item=String> + Send>> {
//         todo!()
//     }
//
//     fn create_dir(&self, path: &str) -> VfsResult<()> {
//         todo!()
//     }
//
//     fn open_file(&self, path: &str) -> VfsResult<Box<dyn SeekAndRead + Send>> {
//         todo!()
//     }
//
//     fn create_file(&self, path: &str) -> VfsResult<Box<dyn Write + Send>> {
//         todo!()
//     }
//
//     fn append_file(&self, path: &str) -> VfsResult<Box<dyn Write + Send>> {
//         todo!()
//     }
//
//     fn metadata(&self, path: &str) -> VfsResult<VfsMetadata> {
//         todo!()
//     }
//
//     fn exists(&self, path: &str) -> VfsResult<bool> {
//         todo!()
//     }
//
//     fn remove_file(&self, path: &str) -> VfsResult<()> {
//         todo!()
//     }
//
//     fn remove_dir(&self, path: &str) -> VfsResult<()> {
//         todo!()
//     }
// }

