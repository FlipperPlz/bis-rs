use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PreprocessorReadError {
    #[error(transparent)]
    IO(#[from] io::Error)
}