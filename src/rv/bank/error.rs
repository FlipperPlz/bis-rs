use std::error::Error;
#[derive(Debug)]
pub enum EntryError {
    EntryNotFound,
}

impl std::fmt::Display for EntryError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            EntryError::EntryNotFound => write!(f, "Entry not found in the current context."),
        }
    }
}

impl Error for EntryError {}