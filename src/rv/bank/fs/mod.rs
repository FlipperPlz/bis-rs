mod error;

use std::collections::HashMap;
use std::fmt::{Debug};
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{Write};
use std::path::Path;
use async_std::io as aio;
use vfs::{FileSystem, SeekAndRead};
use crate::{BankEntry, HEADER_PREFIX_MAGIC, PboFileSkim, PboReader};
use crate::fs::error::BankLoadError;
use crate::options::BankSkimOptions;


#[derive(Debug)]
struct BankFilesystem {
    banks:    Vec<(BankFileMeta, PboFileSkim<File>)>,
}

#[derive(Debug)]
struct BankFileMeta {
    prefix:            String,
    rewritten_entries: HashMap<String, CachedEntry>,
    deleted_entries:   HashMap<String, String>,
    renamed_entries:   HashMap<String, String>,
}

#[derive(Debug)]
struct CachedEntry {
    meta: BankEntry,
    data: Box<aio::Cursor<Vec<u8>>>
}

impl BankFilesystem {
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

        self.banks.push((BankFileMeta::new(prefix), archive));
        Ok(())
    }

    pub fn open_bank(&self, path: &str) {
        todo!()
    }
}

impl BankFileMeta {
    fn new(prefix: String) -> Self {
        Self {
            prefix,
            rewritten_entries: HashMap::new(),
            deleted_entries: HashMap::new(),
            renamed_entries: HashMap::new(),
        }
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

