use std::io::{Read, Write};

pub trait Binarizable {
    fn binarize (
        &self,
        writer: &mut impl Write
    ) -> Result<(), Box<dyn std::error::Error>>;
}

pub trait Debinarizable {
    fn binarize (
        writer: &mut impl Read
    ) -> Result<Self, Box<dyn std::error::Error>>;
}