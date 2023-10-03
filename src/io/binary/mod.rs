use std::error::Error;
use std::io::{Read, Seek, Write};

pub trait Binarizable {
    fn binarize (
        &self,
        writer: &mut impl Write
    ) -> Result<(), Box<dyn std::error::Error>>;
}

pub trait Debinarizable<TError : Error>: Sized {
    fn debinarize (
        reader: &mut (impl Read + Seek),
    ) -> Result<Self, Box<TError>>;
}

