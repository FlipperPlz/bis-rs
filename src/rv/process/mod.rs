pub mod io; pub use io::*;

use std::io::{Read, Seek, Write};
use thiserror::Error;
use vfs::FileSystem;

struct Preprocessor {
    filesystem: Box<dyn FileSystem>,
    current_item: LexToken,
    current_line: u32,
    current_file: String
}

#[derive(Error, Debug)]
pub enum PreprocessError {

}

impl Preprocessor {

    pub fn process<I: Read + Seek, O: Write>(&mut self, output: Option<O>, path: String) -> Result<(), PreprocessError> {
        self.current_line = 0;
        self.current_file = path;
        let mut input: PreprocessorReader<I> = PreprocessorReader::new(self.follow_include(&self.current_file)?);
        self.current_item = LexToken::NewFile;
        self.global_scan(input, output)
    }

    fn global_scan<I: Read + Seek, O: Write>(&self, mut reader: PreprocessorReader<I>, output: Option<O>) -> Result<(), PreprocessError> {
        todo!()
    }

    fn follow_include<I: Read + Seek>(&self, path: &String) -> Result<I, PreprocessError> {

        todo!()
    }



}