use std::io::{Read, Seek, Write};

pub trait Binarizable {
    fn binarize (
        &self,
        writer: &mut impl Write
    ) -> Result<(), Box<dyn std::error::Error>>;
}

pub trait Debinarizable : Sized {
    fn debinarize (
        reader: &mut impl Read + Seek
    ) -> Result<Self, Box<dyn std::error::Error>>;
}

pub fn read_utf8z(reader: &mut impl Read) -> String{
    let mut bytes = Vec::new();
    loop {
        let mut byte = [0; 1];
        reader.read_exact(&mut byte)?;
        if byte[0] == 0 {
            break;
        }
        bytes.push(byte[0]);
    }
    String::from_utf8(bytes)?
}