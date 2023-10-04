use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::io::{Read};
use byteorder::{LittleEndian, ReadBytesExt};
use log::debug;
use vfs::SeekAndRead;
use crate::rv::bank::{BankFsImpl, PROPERTY_PREFIX};


const MAGIC_DECOMPRESSED: i32    = 0x00000000;
const MAGIC_COMPRESSED:   i32    = 0x43707273;
const MAGIC_ENCRYPTED:    i32    = 0x456e6372;
const MAGIC_VERSION:      i32    = 0x56657273;


#[derive(Clone)]
struct EntryInfo {
    filename:          String,
    mime:              EntryMime,
    original_size:     i32,
    deprecated_offset: i64,
    timestamp:         i32,
    packed_size:       i32,
}

pub enum OffsetLocationStrategy {
    Deprecated,
    Calculate
}


#[derive(PartialEq)]
pub enum ErrorStrategy {
    Allow,
    Deny,
    Ignore
}


pub enum MultipleVersionStrategy {
    Forbid,
    Allow {
        should_read_props:       bool

    }
}

pub struct BankReadOptions {
    multiple_version_strategy:  MultipleVersionStrategy,
    offset_location_strategy:   OffsetLocationStrategy,
    allow_offsets_to_header:    ErrorStrategy,
    require_version_first:      bool,
    require_version_entry:      bool,
    require_version_blanked:    bool,
    ignore_unused_properties:   bool,
    max_entry_count:            usize,
    max_entry_name_length:      usize,
    remove_empty_entries:       bool,
    allow_obfuscated:           bool,
    require_valid_checksum:     bool,
    max_property_length:        [usize; 2]
}

#[derive(PartialEq, Clone)]
enum EntryMime {
    Decompressed,
    Version,
    Compressed,
    Encrypted
}

impl EntryInfo {
    #[inline]
    fn read_entry_meta(reader: &mut impl Read, options: &BankReadOptions) -> Result<EntryInfo, Box<dyn Error>> {
        let filename = read_utf8z(reader, options.max_entry_name_length);
        let mime: EntryMime = EntryMime::from(reader.read_i32::<LittleEndian>()?);
        let original_size = reader.read_i32::<LittleEndian>()?;
        let deprecated_offset = reader.read_i32::<LittleEndian>()? as i64;
        let timestamp = reader.read_i32::<LittleEndian>()?;
        let packed_size = reader.read_i32::<LittleEndian>()?;

        let entry = EntryInfo {
            filename,
            mime,
            original_size,
            deprecated_offset,
            timestamp,
            packed_size,
        };

        Ok(entry)
    }
    #[inline]
    fn is_blank(&self) -> bool {
        self.filename.is_empty() &&
            self.original_size == 0 && self.deprecated_offset == 0 && self.timestamp == 0 &&
            self.packed_size == 0
    }

    fn is_fully_blank(&self) -> bool {
        self.is_blank() && self.mime == EntryMime::Decompressed
    }
}

impl BankFsImpl {
    pub fn debinarize(name: &str, reader: &mut impl SeekAndRead, options: BankReadOptions) -> Result<Self, BankDebinarizationError> {
        let mut properties: HashMap<String, String> = HashMap::new();
        let header_start = reader.stream_position().unwrap();
        debug!("Starting to read bank file at position {}", header_start);
        let (data_entries, calculated_buffer_end, header_end) = {
            let mut version_found: bool = false;
            let mut entries: Vec<EntryInfo> = vec![];
            let mut buffer_end: i64 = header_start as i64;

            //Read First Entry
            {
                let first_entry = EntryInfo::read_entry_meta(reader, &options)?;
                if let EntryMime::Version = first_entry.mime {
                    debug!("First entry was version. ");

                    if options.require_version_blanked && !first_entry.is_blank() {
                        return Err(BankDebinarizationError::VersionNotBlanked);
                    }
                    version_found = true;
                    read_properties(reader, &mut properties, &options)
                } else if options.require_version_first {
                    return Err(BankDebinarizationError::FirstNotVersion);
                } else {
                    entries.push(first_entry)
                }
            }

            //Read Remaining Meta
            {
                while entries.len() < options.max_entry_count {
                    let mut current_entry = EntryInfo::read_entry_meta(reader, &options)?;
                    if current_entry.is_fully_blank() { break; }
                    if current_entry.mime == EntryMime::Version {
                        //Options & Errors: What If Version Entry Is Not Zeroed Out?
                        if !version_found {
                            version_found = true;
                            read_properties(reader, &mut properties, &options);
                        }

                        if let MultipleVersionStrategy::Allow {
                            should_read_props
                        } = options.multiple_version_strategy {
                            if should_read_props {
                                read_properties(reader, &mut properties, &options)
                            }
                        } else { return Err(BankDebinarizationError::VersionNotFound); }
                        current_entry.filename = BankFsImpl::normalize_path(&*current_entry.filename, false);
                        continue;
                    }

                    if options.remove_empty_entries && current_entry.packed_size == 0 {
                        continue;
                    }

                    if let OffsetLocationStrategy::Calculate = options.offset_location_strategy {
                        current_entry.deprecated_offset = buffer_end;
                    }

                    buffer_end += current_entry.packed_size as i64;
                    entries.push(current_entry.clone())
                }
            }

            let header_end = reader.stream_position().unwrap();
            let header_length =  header_end - header_start;
            assert!(header_length > 0);

            {
                let mut i = 0;
                while i != entries.len() {
                    let entry = &mut entries[i];
                    if let OffsetLocationStrategy::Calculate = options.offset_location_strategy {
                        entry.deprecated_offset += header_length as i64;
                    }

                    if options.allow_offsets_to_header != ErrorStrategy::Allow && !is_entry_offset_valid(&entry, header_end) {
                        if options.allow_offsets_to_header == ErrorStrategy::Deny {
                            return Err(BankDebinarizationError::ImpossibleDataOffset)
                        }
                        entries.remove(i);
                        continue;
                    }

                    i += 1;
                }
            }

            (entries, buffer_end, header_end)
        };
        debug!("Finished reading header and calculating offsets. Found {} entry(s); Ended at {}.", data_entries.len(), header_end);
        debug!("Assuming offsets were calculated correctly and not altered, the bank buffer should end at {} followed by a checksum", calculated_buffer_end);
        let filesystem = {
            let mut fs = Self {
                name: String::from(name),
                properties,
                files: Default::default(),
            };

            fs
        };

        Ok(filesystem)
    }
}

fn is_entry_offset_valid(entry: &EntryInfo, header_end: u64) -> bool {
    !(entry.deprecated_offset < 0 || entry.packed_size < 0 || entry.deprecated_offset as u64 >= header_end)
}

impl From<i32> for EntryMime {
    fn from(value: i32) -> Self {
        match value {
            MAGIC_DECOMPRESSED => Self::Decompressed,
            MAGIC_COMPRESSED => Self::Compressed,
            MAGIC_ENCRYPTED => Self::Encrypted,
            MAGIC_VERSION => Self::Version,
            _ => panic!("Unknown entry mime! this was not in the script boss, mayday D:")
        }
    }
}


impl Default for MultipleVersionStrategy {
    fn default() -> Self {
        Self::Forbid
    }
}


impl Default for OffsetLocationStrategy {
    fn default() -> Self {
        Self::Calculate
    }
}

impl Default for BankReadOptions {
    fn default() -> Self {
        Self {
            require_version_first: true,
            require_version_entry: true,
            multiple_version_strategy: MultipleVersionStrategy::default(),
            offset_location_strategy: OffsetLocationStrategy::default(),
            ignore_unused_properties: true,
            max_entry_count: usize::MAX,
            max_entry_name_length: 1024,
            remove_empty_entries: true,
            allow_obfuscated: false,
            require_valid_checksum: true,
            max_property_length: [1024, 1024],
            allow_offsets_to_header: ErrorStrategy::Deny,
            require_version_blanked: true,
        }
    }
}



fn read_properties(reader: &mut impl SeekAndRead, properties: &mut HashMap<String, String>, options: &BankReadOptions ) {
    debug!("Starting to read properties at {}.", reader.stream_position().unwrap());
    loop {
        let name = read_utf8z(reader, options.max_property_length[0]);
        if name.is_empty() {
            debug!("Finishing properties reading at {}.", reader.stream_position().unwrap());
            return;
        }
        let value = {
            let mut value = read_utf8z(reader, options.max_property_length[1]);
            if name == PROPERTY_PREFIX {
                value = BankFsImpl::normalize_path(&*value, true)
            }

            value
        };
        properties.insert(name, value);
    }


}

fn read_utf8z(reader: &mut impl Read, cool_down: usize) -> String{
    let mut bytes = Vec::new();
    while bytes.len() < cool_down {
        let mut byte = [0; 1];
        reader.read_exact(&mut byte).unwrap();
        if byte[0] == 0 {
            break;
        }
        bytes.push(byte[0]);
    }
    String::from_utf8(bytes).unwrap()
}

#[derive(Debug)]
pub enum BankDebinarizationError {
    StringTooLong,
    FirstNotVersion,
    VersionNotFound,
    MultipleVersionsFound,
    ImpossibleDataOffset,
    DecompressionError,
    VersionNotBlanked,
    DecryptionError,
    InvalidChecksum,
    Obfuscated,
    Other(String)
}
impl From<Box<dyn Error>> for BankDebinarizationError {
    fn from(err: Box<dyn Error>) -> BankDebinarizationError {
        BankDebinarizationError::Other(format!("An error occurred: {}", err))
    }
}

impl Display for BankDebinarizationError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            BankDebinarizationError::StringTooLong =>
                write!(f, "Bank Debinarization Error: A string has exceeded the maximum length defined in the debinarization options."),
            BankDebinarizationError::FirstNotVersion =>
                write!(f, "Bank Debinarization Error: The current options are configured to require a version entry to be the first in the bank. "),
            BankDebinarizationError::VersionNotFound =>
                write!(f, "Bank Debinarization Error: The current options are configured to require a version entry and none were found. "),
            BankDebinarizationError::ImpossibleDataOffset =>
                write!(f, "Bank Debinarization Error: The current options are configured to error out when offsets are invalidated. "),
            BankDebinarizationError::DecompressionError =>
                write!(f, "Bank Debinarization Error: There was an error while decompressing an entry. "),
            BankDebinarizationError::DecryptionError =>
                write!(f, "Bank Debinarization Error: There was an error while decrypting an entry. "),
            BankDebinarizationError::InvalidChecksum =>
                write!(f, "Bank Debinarization Error: The checksum does not match the one calculated. "),
            BankDebinarizationError::Obfuscated =>
                write!(f, "Bank Debinarization Error: The options are configured to prohibit obfuscated banks. "),
            BankDebinarizationError::MultipleVersionsFound =>
                write!(f, "Bank Debinarization Error: Multiple version entries were found within the bank supplied, this is configured to be disabled. "),
            BankDebinarizationError::VersionNotBlanked =>
                write!(f, "Bank Debinarization Error: Version entry found with additional info, this is configured to throw an error. "),
            BankDebinarizationError::Other(s) =>
                write!(f, "Bank Debinarization Error: {}", s),

        }
    }
}

impl Error for BankDebinarizationError {

}

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::BufReader;
    use log::error;
    use  super::*;

    #[cfg(test)]
    fn setup_logging() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_debinarize() {
        setup_logging();
        let file = File::open("C:\\Users\\ryann\\Desktop\\Testing\\offset_trap.pbo").expect("unable to open file");
        let mut reader = BufReader::new(file);
        let bank_options = BankReadOptions::default();
        match BankFsImpl::debinarize("offset_trap", &mut reader, bank_options) {
            Ok(bank) => {}
            Err(error) => { error!("Error occurred: {}", error); }
        };
    }
}