use std::io::{Read, Seek, SeekFrom};
use crate::PredicateOption;
use crate::preproc::error::PreprocessorReadError;

const BUFF_SIZE: usize = 16;

type ReaderResult<T> = Result<T, PreprocessorReadError>;

struct PreprocessorReader<R: Read + Seek> {
    reader:                 R,
    proc_newline_count:     u32,
    buffer_start:           usize, //the index in our reader where buffer starts
    buffer_index:           usize, // will never be longer than BUFF_SIZE
    buffer:                 [u8; BUFF_SIZE],
    actual_buffer_length:   usize //the amount of bytes loaded into the buffer on last read
}

impl<R: Read + Seek> PreprocessorReader<R> {
    pub fn get(&mut self, stripped: bool) -> ReaderResult<u8> {
        if stripped { self.next_stripped_character() } else { self.next_character() }
    }

    pub fn scan_name(&mut self) -> ReaderResult<String> {
        let mut first = true;
        Ok(self.next_while(true, |next| {
            let state: PredicateOption<()> = match Self::valid_identifier_char(next, first) {
                true => PredicateOption::Continue,
                false => PredicateOption::Exit,
            };
            if first { first = false; }
            state
        })?)
    }

    // pub fn scan_string(max_length: u32, terminators: &Vec<u8>) -> Option<String> {
    //
    // }

    fn next_stripped_character(&mut self) -> ReaderResult<u8> {
        let mut current = self.next_character()?;
        while current == b'\r' { current = self.next_character()? }
        while current == b'\\' {
            {
                let mut next = self.next_character()?;
                while next != b'\r' { next = self.next_character()? };
                if next != b'\n' { self.un_get(); return Ok(current); };
            }
            current = self.next_character()?;
            self.proc_newline_count += 1;
            while current == b'\r' { current = self.next_character()? }
        }

        Ok(current)
    }

    fn next_character(&mut self) -> ReaderResult<u8> {
        if self.buffer_index >= self.actual_buffer_length { self.fill_buffer(); }

        let char: u8  = *self.buffer.get(self.buffer_index)?;
        self.buffer_index += 1;
        Ok(char)
    }



    pub fn new(reader: R) -> Self {
        let mut it = Self {
            reader,
            proc_newline_count: 0,
            buffer_start: 0,
            buffer_index: 0,
            buffer: [0; BUFF_SIZE],
            actual_buffer_length: 0,
        };
        it.fill_buffer();

        it
    }

    pub fn pos(&self) -> usize { self.buffer_start + self.buffer_index }

    fn valid_identifier_char(char: &u8, is_first: bool) -> bool {
        match char {
            b'0'..=b'9' => !is_first,
            b'a'..=b'z' => true,
            _ => false
        }
    }

    fn next_while<E>(&mut self, use_stripped: bool, predicate: impl Fn(&mut u8) -> PredicateOption<E>) -> Result<String, E> {
        let mut string = String::new();
        loop {
            let mut peeked = self.get(use_stripped)?;
            match predicate(&mut peeked) {
                PredicateOption::Skip => { continue }
                PredicateOption::Continue => {string.push(peeked as char)}
                PredicateOption::Exit => { self.un_get(); return Ok(string)}
                PredicateOption::Err(e) => { self.un_get(); return Err(e) }
            }
            peeked
        }
    }



    fn peek(&mut self, use_stripped: bool, count: usize) -> ReaderResult<u8> {
        let buffer_end = self.buffer_start + self.actual_buffer_length;
        let next_pos = self.buffer_start + self.buffer_index + count;
        return Ok(if next_pos >= buffer_end {
            let next = self.get(use_stripped)?;
            self.reader.seek(SeekFrom::Start(self.pos() as u64)).ok()?;
            next
        } else {
            self.buffer.get(self.buffer_index + count).copied()
        })
    }



    fn fill_buffer(&mut self) {
        self.buffer_start += BUFF_SIZE;
        self.buffer_index = 0;
        match { self.reader.read(&mut *self.buffer) } {
            Ok(it) => self.actual_buffer_length = it,
            Err(_) => None
        }
    }

    fn un_get_multi(&mut self, count: usize) -> ReaderResult<u8> {
        self.buffer_index -= count;
        *self.buffer.get(self.buffer_index)
    }

    fn un_get(&mut self) -> ReaderResult<u8> { self.un_get_multi(1) }

}