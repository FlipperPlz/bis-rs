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
    #[error("Failed to get filename as prefix.")]
    FileNameUnknown
}