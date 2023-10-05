use std::io::{Read, Seek};
use byteorder::{LittleEndian, ReadBytesExt};
use crate::{BankEntry, Debinarizable, EntyMime};
use crate::read::{EntryMetadataError, EntryNameError};

pub struct PboReader<R: Read + Seek> {
    reader: R,
}

impl<R: Read + Seek> PboReader<R> {
    fn read_entry(&mut self) -> Result<BankEntry, EntryMetadataError> {
        return Ok(
            BankEntry {
                filename: self.read_entry_name()?,
                mime: self.read_mime()?,
                size_unpacked: self.reader.read_i32::<LittleEndian>()? as u32,
                start_offset: self.reader.read_i32::<LittleEndian>()? as u64,
                timestamp: self.reader.read_i32::<LittleEndian>()? as u64,
                size_packed: self.reader.read_i32::<LittleEndian>()? as u64,
            }
        )
    }

    #[inline]
    fn read_mime(&mut self) -> Result<EntyMime, EntryMetadataError> {
        return EntyMime::try_from(self.reader.read_i32::<LittleEndian>()?)
    }

    #[inline]
    fn read_entry_name(&mut self) -> Result<String, EntryNameError> {
        let mut vec = Vec::new();

        for _ in 0..crate::ENTRY_NAME_MAX {
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
}

impl<R: Read + Seek> Debinarizable<PboReader<R>> for EntyMime {
    type Error = EntryMetadataError;

    fn debinarize(reader: &mut PboReader<R>) -> Result<Self, Self::Error> {
        reader.read_mime()
    }
}

impl<R: Read + Seek> Debinarizable<PboReader<R>> for BankEntry {
    type Error = EntryMetadataError;

    fn debinarize(reader: &mut PboReader<R>) -> Result<Self, Self::Error> {
        return reader.read_entry()
    }
}
