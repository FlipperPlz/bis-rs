
use std::collections::HashMap;
use std::io::{Read, Seek};
use byteorder::{LittleEndian, ReadBytesExt};
use crate::{BankSkimEntry, Debinarizable, DebinarizationOptions, DebinarizePredicateOption, EntryMime, magic_enum, PboFileSkim};
use std::io;
use thiserror::Error;

const WIN_DIR: char = '\\';
const UNIX_DIR: char = '/';
pub const BANK_DIR: char = UNIX_DIR;
pub const MAX_PATH_LENGTH: u16 = 1023;
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

#[derive(Error, Debug)]
pub enum EntryError {
    #[error("Entry Read Error: The provided offset for the entry is invalid. ")]
    SeekFailed,
    #[error("Entry Read Error: The provided entry was not found in the bank.")]
    EntryNotFound,

}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum OffsetLocationStrategy {
    Deprecated,
    Calculate
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct BankSkimOptions {
    pub(crate) offset_location_strategy:   OffsetLocationStrategy,
    pub(crate) allow_offsets_to_header:    bool,
    pub(crate) remove_impossible_offsets:  bool,
    pub(crate) require_version_first:      bool,
    pub(crate) require_version_entry:      bool,
    pub(crate) ignore_unused_properties:   bool,
    pub(crate) max_entry_count:            usize,
    pub(crate) remove_empty_entries:       bool,
    pub(crate) allow_obfuscated:           bool,
    pub(crate) require_valid_checksum:     bool,
}

impl Default for BankSkimOptions {
    fn default() -> Self {
        Self {
            offset_location_strategy: OffsetLocationStrategy::Deprecated,
            allow_offsets_to_header: false,
            remove_impossible_offsets: false,
            require_version_first: false,
            require_version_entry: false,
            ignore_unused_properties: false,
            max_entry_count: usize::MAX,
            remove_empty_entries: false,
            allow_obfuscated: false,
            require_valid_checksum: false,
        }
    }
}

impl DebinarizationOptions for BankSkimOptions {

}



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
    #[error("Invalid Encryption")]
    EncryptionError(
        #[from] BankEncryptionError
    ),
    #[error(transparent)]
    IO(
        #[from] io::Error
    )
}

#[derive(Debug, Error)]
pub enum BankEncryptionError {
    #[error("The encryption format of this pbo is not supported")]
    NotSupported
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

pub const HEADER_PREFIX_MAGIC: &str = "prefix";
const HEADER_ENCRYPTION_MAGIC: &str = "hprotect";
const SERIAL_MAGIC: &str = "registry";
const PADDING_NAME: &str = "___dummypadding___";
const ENCRYPTION_MAGIC: &str = "encryption";


pub enum EncryptionType {
    Header {
        version: i32
    },
    Data {
        headers_size: i32,
        encoded_headers_size: i32
    },
    None
}

pub fn get_encryption_mode<R: Read + Seek>(reader: &mut PboReader<R>, properties: &HashMap<String, String>) -> Result<EncryptionType, EntryMetadataError> {

    // match properties.get(HEADER_ENCRYPTION_MAGIC) {
    //     None => {}
    //     Some(value) => {
    //         let version = reader.read_i32::<LittleEndian>()?;
    //     }
    // }
    Ok(EncryptionType::None)
}


#[derive(Clone, Debug)]
pub struct PboReader<R: Read + Seek> {
    reader:   R,
    position: u64
}

impl<R: Read + Seek> PboReader<R> {
    #[inline]
    pub fn skim_archive(reader: R, options: BankSkimOptions) -> Result<PboFileSkim<R>, BankSkimError> {
        let mut reader = PboReader { reader, position: 0 };
        let (properties, entries) = reader.process_entries(&options)?;


        Ok(PboFileSkim::<R> {
            reader,
            entries,
            options,
            properties,
        })
    }

    pub fn read_entry_data(&mut self, entry: &BankSkimEntry, offset: &u64) -> Result<Vec<u8>, EntryError> {
        // if self.reader.seek(SeekFrom::Start(*offset)).or_else(EntryError::SeekFailed)? == *offset {
        //     //TODO: read compressed
        // }
        Err(EntryError::SeekFailed)
    }

    ///This function does some processing on the embedded entries in the bank file, and all though
    /// there is an offset stored in the file itself, it's not used in the newer games and has since
    /// been deprecated as a waste of space.
    ///
    /// Notes:
    /// When not using the deprecated offsets, the offsets are calculated it a pretty terrible way.
    /// In order to support this we end up doing all sorts of up/down casting.
    #[inline]
    fn process_entries(&mut self, options: &BankSkimOptions) -> Result<(HashMap<String, String>, HashMap<BankSkimEntry, u64>), BankSkimError> {
        let mut properties = HashMap::new();
        let entries: HashMap<BankSkimEntry, u64>;
        let end_of_bank: i32;
        let buffer_start: u64;
        {
            let mut e_offset: i32 = 0;
            let mut first: bool = true;
            let closure_entries = BankSkimEntry::debinarize_while(self, |e, closure_reader| {
                match options.offset_location_strategy {
                    OffsetLocationStrategy::Calculate => {
                        e.start_offset = e_offset as u64;
                        e_offset += e.size_packed as i32;
                    }
                    _ => {}
                }
                return if e.start_offset <= 0 && options.remove_impossible_offsets {
                    first = false;
                    Ok(DebinarizePredicateOption::Skip)
                } else {
                    if empty_name(e) {
                        return if is_version(e) {
                            if first {
                                closure_reader.read_properties(&mut properties)?;
                                match get_encryption_mode::<R>(closure_reader, &properties)? {
                                    EncryptionType::Header { .. } => {todo!()}
                                    EncryptionType::Data { .. } => {todo!()}
                                    EncryptionType::None => {}
                                }
                            }
                            first = false;
                            Ok(DebinarizePredicateOption::Ok)
                        } else {
                            first = false;
                            Ok(DebinarizePredicateOption::Break)
                        }
                    } else { first = false; }
                    e.filename = convert_dir_slash(&e.filename);
                    Ok(DebinarizePredicateOption::Ok)
                }
            })?;
            buffer_start = self.position;
            entries = closure_entries.into_iter().filter_map(|e| {
                let start = e.start_offset.clone() + buffer_start;

                if options.allow_offsets_to_header || start >= buffer_start {
                    Some((e, start))
                } else { None }
            }).collect();
            end_of_bank = e_offset;
        }
        Ok((properties, entries))
    }

    #[inline]
    fn read_file_info(&mut self) -> Result<(bool, BankSkimEntry), BankSkimError> {
        let entry = self.read_entry()?;
        return Ok((is_version(&entry), entry))
    }

    #[inline]
    fn read_entry(&mut self) -> Result<BankSkimEntry, EntryMetadataError> {
        return Ok(
            BankSkimEntry {
                filename: self.read_entry_name()?,
                mime: self.read_mime()?,
                size_unpacked: self.read_int()? as u32,
                start_offset: self.read_int()? as u64,
                timestamp: self.read_int()? as u32,
                size_packed: self.read_int()? as u32,
            }
        )
    }


    #[inline]
    fn read_int(&mut self) -> Result<i32, io::Error> {
        let val = self.reader.read_i32::<LittleEndian>()?;
        self.position += 4;
        Ok(val)
    }

    #[inline]
    fn read_mime(&mut self) -> Result<EntryMime, EntryMetadataError> {
        EntryMime::try_from(self.read_int()?)
    }

    #[inline]
    fn read_entry_name(&mut self) -> Result<String, EntryNameError> {
        let mut vec = Vec::new();

        for _ in 0..MAX_PATH_LENGTH {
            self.position += 1;
            match self.reader.read_u8()? as i32 {
                0 => break,
                i if i < 0 => {
                    return Err(EntryNameError::Underflow)
                },
                current => vec.push(current as u8),
            };
        }

        Ok(String::from_utf8(vec.to_owned()).unwrap().to_lowercase())
    }

    #[inline]
    fn read_properties(&mut self, properties: &mut HashMap<String, String>) -> Result<(), EntryMetadataError> {
        loop {
            let name = self.read_entry_name()?;

            if name.is_empty() { break }

            let value =if name.eq(HEADER_PREFIX_MAGIC) {
                let mut it = self.read_entry_name()?;
                it.push(BANK_DIR);
                it
            } else { self.read_entry_name()? };
            properties.insert(name, value);
        }
        Ok(())
    }
}

impl<R: Read + Seek> Debinarizable<PboReader<R>> for EntryMime {
    type Error = EntryMetadataError;

    fn debinarize(reader: &mut PboReader<R>) -> Result<Self, Self::Error> {
        reader.read_mime()
    }
}

impl<R: Read + Seek> Debinarizable<PboReader<R>> for BankSkimEntry {
    type Error = EntryMetadataError;

    fn debinarize(reader: &mut PboReader<R>) -> Result<Self, Self::Error> {
        reader.read_entry()
    }
}

#[inline]
fn is_version(entry: &BankSkimEntry) -> bool {
    entry.mime == EntryMime::Version && entry.size_packed == 0 && entry.timestamp == 0
}

#[inline]
fn empty_name(entry: &BankSkimEntry) -> bool {
    entry.filename.is_empty()
}

#[inline]
pub fn normalize_path(path: &str, directory: bool) -> String {
    if path.is_empty() {
        return path.to_string();
    }

    let mut result = Vec::with_capacity(path.len());
    let mut last_was_separator = true;

    for c in path.chars() {
        match c {
            UNIX_DIR | WIN_DIR => {
                if last_was_separator { continue }

                result.push(WIN_DIR);
                last_was_separator = true;
            },
            _ => {
                last_was_separator = false;
                result.push(c.to_ascii_lowercase());
            }
        }
    }

    if !directory && !result.is_empty() && *result.last().unwrap() == WIN_DIR {
        result.pop();
    }

    result.iter().collect()
}

#[inline]
pub fn convert_dir_slash(name: &String) -> String {
    if !name.contains(UNIX_DIR) {
        return name.clone();
    }

    name.replace(UNIX_DIR, "\\")
}