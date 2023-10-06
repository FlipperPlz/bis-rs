mod error;

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fmt::{Debug};
use std::fs::File;
use std::io::{Write};
use std::path::Path;
use io_streams::BufDuplexer;
use vfs::{FileSystem, SeekAndRead, VfsMetadata, VfsResult};
use crate::{HEADER_PREFIX_MAGIC, PboFileSkim, PboReader};
use crate::fs::error::BankLoadError;
use crate::options::BankSkimOptions;


#[derive(Debug)]
struct BankFilesystem {
    // banks:    HashMap<String, PboFileSkim<BufDuplexer<File>>>,

}

impl BankFilesystem {
    //
    // pub fn load_bank(&mut self, path: &Path, options: BankSkimOptions) -> Result<(), BankLoadError> {
    //     let file = File::open(path)?;
    //     let archive = PboReader::skim_archive(BufDuplexer::new(file), options)?;
    //     let prefix = match archive.properties.get(HEADER_PREFIX_MAGIC) {
    //         None => match path.file_stem() {
    //             None => Err(BankLoadError::FileNameUnknown),
    //             Some(it) => Ok(
    //                 it.to_str().ok_or_else(BankLoadError::FileNameUnknown).to_string()
    //             )
    //         }
    //         Some(it) => Ok(it.clone())
    //     }?;
    //
    //     self.banks.insert(prefix, archive);
    //     Ok(())
    // }
}



impl FileSystem for BankFilesystem {

    fn read_dir(&self, path: &str) -> VfsResult<Box<dyn Iterator<Item=String> + Send>> {
        todo!()
    }

    fn create_dir(&self, path: &str) -> VfsResult<()> {
        todo!()
    }

    fn open_file(&self, path: &str) -> VfsResult<Box<dyn SeekAndRead + Send>> {
        todo!()
    }

    fn create_file(&self, path: &str) -> VfsResult<Box<dyn Write + Send>> {
        todo!()
    }

    fn append_file(&self, path: &str) -> VfsResult<Box<dyn Write + Send>> {
        todo!()
    }

    fn metadata(&self, path: &str) -> VfsResult<VfsMetadata> {
        todo!()
    }

    fn exists(&self, path: &str) -> VfsResult<bool> {
        todo!()
    }

    fn remove_file(&self, path: &str) -> VfsResult<()> {
        todo!()
    }

    fn remove_dir(&self, path: &str) -> VfsResult<()> {
        todo!()
    }
}

