use std::collections::HashMap;
use std::io::{Read, Seek};
use byteorder::{LittleEndian, ReadBytesExt};
use crate::{
    BankEntry,
    BankFile,
    BankSkimError,
    BankSkimOptions,
    CustomDebinarizable,
    Debinarizable,
    DebinarizePredicateOption,
    EntryMetadataError,
    EntryNameError,
    EntyMime,
    OffsetLocationStrategy,
    path
};



pub struct PboReader<R: Read + Seek> {
    reader: R,
}

impl<R: Read + Seek> PboReader<R> {
    #[inline]
    fn read_archive(reader: &mut R, options: BankSkimOptions) -> Result<BankFile, BankSkimError> {
        let header_start = reader.stream_position().unwrap();
        let (properties, entries) = Self::process_entries(reader, options)?;
        let buffer_start = reader.stream_position().unwrap();

        Ok(BankFile {
            header_start,
            buffer_start,
            entries,
            properties,
        })
    }

    #[inline]
    fn process_entries(reader: &mut R, options: BankSkimOptions) -> Result<(HashMap<String, String>, Vec<BankEntry>), BankSkimError> {
        let mut e_offset: i32 = 0;
        let mut last_was_version: bool = false;
        let mut properties = HashMap::new();
        let entries = BankEntry::debinarize_while(reader, |e, closure_reader| {
            match options.offset_location_strategy {
                OffsetLocationStrategy::Calculate => {
                    e.start_offset = e_offset as u64;
                    e_offset += e.size_packed as i32;

                    if e.start_offset <= 0 && options.remove_impossible_offsets {
                        return Ok(DebinarizePredicateOption::Skip)
                    }
                }
                _ => {}
            }

            if empty_name(e) {
                return if is_version(e) {
                    last_was_version = true;
                    Self::read_properties(closure_reader, &mut properties)?;
                    Ok(DebinarizePredicateOption::Ok)
                } else { Ok(DebinarizePredicateOption::Break) };
            }
            e.filename = path::convert_dir_slash(&e.filename);
            Ok(DebinarizePredicateOption::Ok)
        })?;

        Ok((properties, entries))
    }

    #[inline]
    fn read_file_info(reader: &mut R) -> Result<(bool, BankEntry), BankSkimError> {
        let entry = Self::read_entry(reader)?;
        return Ok((is_version(&entry), entry))
    }


    #[inline]
    fn read_entry(reader: &mut R, ) -> Result<BankEntry, EntryMetadataError> {
        return Ok(
            BankEntry {
                filename: Self::read_entry_name(reader)?,
                mime: Self::read_mime(reader)?,
                size_unpacked: reader.read_i32::<LittleEndian>()? as u32,
                start_offset: reader.read_i32::<LittleEndian>()? as u64,
                timestamp: reader.read_i32::<LittleEndian>()? as u64,
                size_packed: reader.read_i32::<LittleEndian>()? as u64,
            }
        )
    }

    #[inline]
    fn read_mime(reader: &mut R, ) -> Result<EntyMime, EntryMetadataError> {
        return EntyMime::try_from(reader.read_i32::<LittleEndian>()?)
    }

    #[inline]
    fn read_entry_name(reader: &mut R) -> Result<String, EntryNameError> {
        let mut vec = Vec::new();

        for _ in 0..crate::ENTRY_NAME_MAX {
            match reader.read_u8()? as i32 {
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
    fn read_properties(reader: &mut R, properties: &mut HashMap<String, String>) -> Result<(), EntryMetadataError> {
        loop {
            let name = Self::read_entry_name(reader)?;
            if name.is_empty() { break }
            let value = Self::read_entry_name(reader)?;
            properties.insert(name, value);
        }
        Ok(())
    }
}

impl<R: Read + Seek> Debinarizable<R> for EntyMime {
    type Error = EntryMetadataError;

    fn debinarize(reader: &mut R) -> Result<Self, Self::Error> {
        PboReader::read_mime(reader)
    }
}

impl<R: Read + Seek> Debinarizable<R> for BankEntry {
    type Error = EntryMetadataError;

    fn debinarize(reader: &mut R) -> Result<Self, Self::Error> {
        PboReader::read_entry(reader)
    }
}

impl<R: Read + Seek> CustomDebinarizable<R, BankSkimOptions> for BankFile {
    fn debinarize_with_options(reader: &mut R, options: BankSkimOptions) -> Result<Self, Self::Error> {
        PboReader::read_archive(reader, options)
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