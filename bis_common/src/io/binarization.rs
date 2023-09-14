pub trait BinaryObject<BinarizationOptions, DebinarizationOptions> : Binarizable<BinarizationOptions> + Debinarizable<DebinarizationOptions>{}

pub trait StrictBinaryObject<BinarizationOptions, DebinarizationOptions, ValidationOptions> : BinaryObject<BinarizationOptions, DebinarizationOptions> + Validatable<ValidationOptions>{}
pub trait Debinarizable<DebinarizationOptions> : Sized {
    fn debinarize (
        reader: &mut impl std::io::Read,
         options: DebinarizationOptions
    ) -> Result<Self, Box<dyn std::error::Error>>;
}

pub trait Binarizable<BinarizationOptions> {
    fn binarize (
        &self,
        writer: &mut impl std::io::Write,
        options: BinarizationOptions
    ) -> Result<(), Box<dyn std::error::Error>>;
}

pub trait Validatable<ValidationOptions> {

    fn validate (
        &self,
        options: ValidationOptions
    ) -> Result<(), Box<dyn std::error::Error>>;
}