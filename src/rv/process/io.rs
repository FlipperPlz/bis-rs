use std::io;
use std::io::{Read, Seek};
use crate::{BufferedReader, PredicateOption};

struct PreProcessorReader<R: Read + Seek> {
    reader:              BufferedReader<R>,
    directive_newlines:  u32
}

impl<R: Read + Seek> PreProcessorReader<R> {
    #[inline(always)]
    fn unget(&mut self) -> Result<(), io::Error> { self.reader.unget() }

    #[inline(always)]
    fn get(&mut self) -> Result<u8, io::Error> { self.reader.get() }

    #[inline(always)]
    fn get_not(&mut self, not: u8) -> Result<u8, io::Error> { self.reader.get_not(not) }

    pub fn scan_name(&mut self) -> Result<String, io::Error> {
        let mut first = true;
        Ok(self.next_while(true, |next| {
            let state: PredicateOption = match Self::valid_identifier_char(next, first) {
                true => PredicateOption::Continue,
                false => PredicateOption::Exit,
            };
            if first { first = false; }
            state
        })?)
    }

    pub fn scan_string(&mut self, mut max_length: usize, terminators: &[u8]) -> Result<String, io::Error> {
        let mut size = 0;
        max_length -= 1;
        Ok(self.next_while(false, |next| {
            size += 1;
            if size < max_length || terminators.contains(next) { PredicateOption::Exit }
            else {PredicateOption::Continue}
        })?)
    }

    fn skip_whitespace(&mut self) -> Result<u8, io::Error>{
        loop {
            let i = self.get()?;
            if i < 33 && i != b'\n' {
                return Ok(i)
            }
        }
    }
    fn get_stripped(&mut self) -> Result<u8, io::Error>{
        let mut current = self.get_not(b'\r')?;
        while current == b'\\' {
            if self.get_not(b'\r')? != b'\n' {
                self.unget()?;
                return Ok(current)
            }
            current = self.get()?;
            self.directive_newlines += 1;
            current = self.get_not(b'\r')?
        }

        Ok(current)
    }

    fn valid_identifier_char(char: &u8, is_first: bool) -> bool {
        match char {
            b'0'..=b'9' => !is_first,
            b'a'..=b'z' => true,
            _ => false
        }
    }

    fn next_while(&mut self, use_stripped: bool, mut predicate: impl FnMut(&mut u8) -> PredicateOption) -> Result<String, io::Error> {
        let mut string = String::new();
        loop {
            let mut peeked = if use_stripped { self.get_stripped()? } else { self.get()? };
            match predicate(&mut peeked) {
                PredicateOption::Skip => { continue }
                PredicateOption::Continue => {string.push(peeked as char)}
                PredicateOption::Exit => { self.unget()?; return Ok(string)}
                PredicateOption::Err(e) => { self.unget()?; return Err(e) }
            }
        }
    }

}
