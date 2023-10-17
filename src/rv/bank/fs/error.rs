use std::io;
use log::error;
use thiserror::Error;
use crate::BankSkimError;

#[derive(Error, Debug)]
pub enum BankLoadError {
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error(transparent)]
    EntryDebinarization(#[from] BankSkimError),
    #[error("Failed to load bank with prefix {0} as a bank is already loaded with the same prefix. ")]
    PreexistingPrefix(String),
    #[error("Failed to get filename as prefix.")]
    FileNameUnknown
}