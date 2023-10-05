use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EntryMetadataError {
    #[error("Bank Debinarization Error: Entry mime not supported: {0}")]
    EntryMimeNotSupported(i32),
    #[error("Bank Debinarization Error: The options are configured to forbid obfuscated entries.")]
    Obfuscated,
    #[error("Invalid Name")]
    EntryNameError(
        #[from] EntryNameError
    ),
    #[error(transparent)]
    IO(
        #[from] io::Error
    )
}
#[derive(Debug, Error)]
pub enum EntryNameError {
    #[error("An entry was found with a weird name. I dont know how to handle this yet or if its possible.")]
    Underflow,
    #[error(transparent)]
    IO(
        #[from] io::Error
    )
}