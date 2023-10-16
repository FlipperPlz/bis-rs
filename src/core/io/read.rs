use std::io;
use std::io::{Read, Seek, SeekFrom};
const BUFF_SIZE: usize = 512;

pub enum PredicateOption {
    Skip,
    Continue,
    Exit,
    Err(io::Error)
}

pub struct BufferedReader<R: Read + Seek> {
    reader:                 R,
    buffer_start:           usize, //the index in our reader where buffer starts
    buffer_index:           usize, // will never be longer than BUFF_SIZE
    buffer:                 [u8; BUFF_SIZE],
    actual_buffer_length:   usize //the amount of bytes loaded into the buffer on last read
}

impl<R: Read + Seek> BufferedReader<R> {

    pub fn new(reader: R) -> BufferedReader<R> {
        let mut r = BufferedReader {
            reader,
            buffer_start: 0,
            buffer_index: 0,
            buffer: [0; BUFF_SIZE],
            actual_buffer_length: 0,
        };
        r.load_buffer().unwrap();
        r
    }

    fn load_buffer(&mut self) -> Result<(), io::Error> {
        self.reader.seek(SeekFrom::Start(self.buffer_start as u64))?;
        self.actual_buffer_length = self.reader.read(&mut self.buffer)?;
        self.buffer_index = 0;
        Ok(())
    }


    pub fn pos(&self) -> usize { self.buffer_start + self.buffer_index }

    pub fn get(&mut self) -> Result<u8, io::Error> {
        if self.buffer_index >= self.actual_buffer_length {
            self.buffer_start += self.actual_buffer_length;
            self.load_buffer()?;
        }

        if self.buffer_index < self.actual_buffer_length {
            let val = self.buffer[self.buffer_index];
            self.buffer_index += 1;
            return Ok(val)
        }

        Err(io::Error::other("Invalid position! {core::io::read::BufferedReader<R>::get(&mut self);}"))
    }

    #[inline]
    pub fn get_not(&mut self, not: u8) -> Result<u8, io::Error> {
        let mut current = self.get()?;
        while current == not { current = self.get()? }
        Ok(current)
    }



    pub fn unget(&mut self) -> Result<(), io::Error>{
        return if self.buffer_index == 0 {
            if self.buffer_start <= 0 {
                Err(io::Error::other("At start of stream cannot unget. {core::io::read::BufferedReader<R>::unget(&mut self);}"))
            } else {
                self.buffer_start -= std::cmp::min(self.buffer_start, BUFF_SIZE);
                self.load_buffer()?;
                Ok(self.buffer_index = self.actual_buffer_length)
            }
        } else if self.buffer_start > 0 {
            Ok(self.buffer_index -= 1)
        } else {
            panic!("Buffer index is less than 0; this is impossible! You broke something :(")
        }

    }

}


