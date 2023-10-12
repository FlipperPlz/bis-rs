
pub enum PredicateOption<E> {
    Skip,
    Continue,
    Exit,
    Err(E)
}