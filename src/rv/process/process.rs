use std::io;
use bex::{Lexer, PreProcess};
use thiserror::Error;
use crate::{ProcLexicalError, ProcToken};

pub struct RvPreprocessor {

}
#[derive(Error, Debug)]
pub enum PreprocessorError {
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error(transparent)]
    Lexer(#[from] ProcLexicalError)
}


impl PreProcess<u8> for RvPreprocessor {
    type E = PreprocessorError;

    fn preprocess(&mut self, lexer: Lexer<u8>) -> Result<Lexer<u8>, Self::E> {
        let contents = lexer.tokenize_until_end::<ProcToken>()?;
        todo!()
    }
}