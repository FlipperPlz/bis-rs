use std::collections::HashMap;
use std::fmt::{Debug};
use std::io::{Write};
use io_streams::StreamDuplexer;
use vfs::{FileSystem, SeekAndRead, VfsMetadata, VfsResult};
use crate::PboFileSkim;


#[derive(Debug)]
struct BankFilesystem {
    banks:    HashMap<String, PboFileSkim<StreamDuplexer>>
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

