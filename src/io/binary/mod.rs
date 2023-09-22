use std::{error::Error, io};

trait Binarizable {
    fn binarize(&self, writer: dyn io::Write) -> Result<(), dyn Error>;
}

trait Debinarizable {
    fn debinarize(writer: dyn io::Write) -> Result<Self, dyn Error>;
}