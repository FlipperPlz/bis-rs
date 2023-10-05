pub trait DebinarizationOptions : Default {

}

pub enum DebinarizePredicateOption {
    Ok,
    Skip,
    Break,
}

pub trait Debinarizable<R: ?Sized>: Sized + Clone {
    type Error;

    fn debinarize(reader: &mut R) -> Result<Self, Self::Error>;

    fn debinarize_all(reader: &mut R, slice: &mut [Self]) -> Result<(), Self::Error> {
        for elem in slice {
            *elem = Self::debinarize(reader)?;
        }
        Ok(())
    }

    fn debinarize_while(reader: &mut R, mut predicate: impl FnMut(&mut Self, &mut R) -> Result<DebinarizePredicateOption, Self::Error>) -> Result<Vec<Self>, Self::Error> {
        let mut vec = Vec::new();

        while let Ok(mut item) = Self::debinarize(reader) {
            match predicate(&mut item, reader)? {
                DebinarizePredicateOption::Skip => { continue }
                DebinarizePredicateOption::Break => { break }
                _ => {vec.push(item.clone())}
            }
        }

        Ok(vec)
    }

    fn debinarize_until(reader: &mut R, mut predicate: impl FnMut(&Self) -> Result<bool, Self::Error>) -> Result<Vec<Self>, Self::Error> {
        let mut vec = Vec::new();

        while let Ok(item) = Self::debinarize(reader) {
            vec.push(item.clone());

            if predicate(&item)? { break; }
        }

        Ok(vec)
    }
}


pub trait CustomDebinarizable<
    R: ?Sized,
    O: DebinarizationOptions
>: Sized + Clone {
    type Error;

    fn debinarize_with_options(reader: &mut R, options: O) -> Result<Self, Self::Error>;

    fn debinarize(reader: &mut R) -> Result<Self, Self::Error> {
        Self::debinarize_with_options(reader, O::default())
    }


}