

pub struct Bank;

pub struct Entry {
    name: str,
    bank: *Bank,
}

trait IEntry {
    fn form_path_string (form_absolute_path: bool) -> Box<str>;
    fn retrieve_original_size () -> u32;
    fn retrieve_written_size () -> usize;
}

impl IEntry for Entry {
    fn form_path_string(form_absolute_path: bool) -> Box<str> {
        todo!()
    }

    fn retrieve_original_size() -> u32 {
        todo!()
    }

    fn retrieve_written_size() -> usize {
        todo!()
    }
}