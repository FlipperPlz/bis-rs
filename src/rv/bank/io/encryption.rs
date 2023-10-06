use std::collections::HashMap;
use std::io::{Read};
use crate::{EntryMetadataError, PboReader};

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

pub fn get_encryption_mode<R: Read>(reader: &mut PboReader<R>, properties: &HashMap<String, String>) -> Result<EncryptionType, EntryMetadataError> {

    // match properties.get(HEADER_ENCRYPTION_MAGIC) {
    //     None => {}
    //     Some(value) => {
    //         let version = reader.read_i32::<LittleEndian>()?;
    //     }
    // }
    Ok(EncryptionType::None)
}