pub mod error;       pub use error::*;
pub mod options;     pub use options::*;
pub mod encryption;  pub use encryption::*;

use std::collections::HashMap;
use std::io;
use std::io::{Read, Seek};
use byteorder::{LittleEndian, ReadBytesExt};
use crate::{BANK_DIR, Debinarizable, DebinarizePredicateOption, magic_enum, MAX_PATH_LENGTH, path};
use crate::options::{BankSkimOptions, OffsetLocationStrategy};

pub const HEADER_PREFIX_MAGIC: &str = "prefix";

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct BankEntry {
    pub(crate) filename:      String,
    pub(crate) mime:          EntyMime,
    pub(crate) size_unpacked: u32,
    pub(crate) start_offset:  u64,
    pub(crate) timestamp:     u32,
    pub(crate) size_packed:   u32,
}

magic_enum! {
    i32,
    EntyMime,
    EntryMetadataError,
    EntryMimeNotSupported {
        Decompressed = 0x00000000,
        Compressed   = 0x43707273,
        Encrypted    = 0x456e6372,
        Version      = 0x56657273
    }
}


#[derive(Clone, Debug)]
pub struct PboFileSkim<R: Read> {
    pub(crate) reader:        PboReader<R>,
    pub(crate) entries:       HashMap<BankEntry, u64>,
    pub(crate) options:       BankSkimOptions,
    pub(crate) properties:    HashMap<String, String>
}


#[derive(Clone, Debug)]
pub struct PboReader<R: Read> {
    reader:   R,
    position: u64
}

impl<R: Read> PboReader<R> {

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

    ///This function does some processing on the embedded entries in the bank file, and all though
    /// there is an offset stored in the file itself, it's not used in the newer games and has since
    /// been deprecated as a waste of space.
    ///
    /// Notes:
    /// When not using the deprecated offsets, the offsets are calculated it a pretty terrible way.
    /// In order to support this we end up doing all sorts of up/down casting.
    #[inline]
    fn process_entries(&mut self, options: &BankSkimOptions) -> Result<(HashMap<String, String>, HashMap<BankEntry, u64>), BankSkimError> {
        let mut properties = HashMap::new();
        let entries: HashMap<BankEntry, u64>;
        let end_of_bank: i32;
        let buffer_start: u64;
        {
            let mut e_offset: i32 = 0;
            let mut first: bool = true;
            let closure_entries = BankEntry::debinarize_while(self, |e, closure_reader| {
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
                    e.filename = path::convert_dir_slash(&e.filename);
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
    fn read_file_info(&mut self) -> Result<(bool, BankEntry), BankSkimError> {
        let entry = self.read_entry()?;
        return Ok((is_version(&entry), entry))
    }

    #[inline]
    fn read_entry(&mut self) -> Result<BankEntry, EntryMetadataError> {
        return Ok(
            BankEntry {
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
    fn read_mime(&mut self) -> Result<EntyMime, EntryMetadataError> {
        EntyMime::try_from(self.read_int()?)
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

impl<R: Read + Seek> Debinarizable<PboReader<R>> for EntyMime {
    type Error = EntryMetadataError;

    fn debinarize(reader: &mut PboReader<R>) -> Result<Self, Self::Error> {
        reader.read_mime()
    }
}

impl<R: Read> Debinarizable<PboReader<R>> for BankEntry {
    type Error = EntryMetadataError;

    fn debinarize(reader: &mut PboReader<R>) -> Result<Self, Self::Error> {
        reader.read_entry()
    }
}

#[inline]
fn is_version(entry: &BankEntry) -> bool {
    entry.mime == EntyMime::Version && entry.size_packed == 0 && entry.timestamp == 0
}

#[inline]
fn empty_name(entry: &BankEntry) -> bool {
    entry.filename.is_empty()
}