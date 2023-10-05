use thiserror::Error;
use crate::read::EntryMetadataError;

#[derive(Error, Debug)]
pub enum BankSkimError {
    #[error("Bank Debinarization Error: The current options are configured to require a version entry to be the first in the bank.")]
    FirstNotVersion,
    #[error("Bank Debinarization Error: The current options are configured to require a version entry but none were found.")]
    VersionNotFound,
    #[error("Bank Debinarization Error: Multiple version entries were found within the bank supplied, but this is configured to be disabled.")]
    MultipleVersionsFound,
    #[error("Bank Debinarization Error: The current options are configured to error out when offsets are invalidated.")]
    ImpossibleDataOffset,
    #[error("Bank Debinarization Error: Version entry found with additional info, this is configured to throw an error.")]
    VersionNotBlanked,
    #[error("Bank Debinarization Error: The checksum does not match the one calculated.")]
    InvalidChecksum,
    #[error("Bank Debinarization Error: The options are configured to forbid obfuscated banks.")]
    Obfuscated,
    #[error(transparent)]
    EntryDebinarization(#[from] EntryMetadataError),
}