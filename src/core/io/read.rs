use std::io;
use std::io::{Read, Seek, SeekFrom};
const BUFF_SIZE: usize = 512;

struct BufferedReader<R: Read + Seek> {
    reader:                 R,
    buffer_start:           usize, //the index in our reader where buffer starts
    buffer_index:           usize, // will never be longer than BUFF_SIZE
    buffer:                 [u8; BUFF_SIZE],
    actual_buffer_length:   usize //the amount of bytes loaded into the buffer on last read
}

impl<R: Read + Seek> BufferedReader<R> {
    fn load_buffer(&mut self) -> Result<(), io::Error> {
        self.reader.seek(SeekFrom::Start(self.buffer_start as u64))?;
        self.actual_buffer_length = self.reader.read(&mut self.buffer)?;
        self.buffer_index = 0;
        Ok(())
    }


    pub fn get(&mut self) -> Result<u8, io::Error> {
        if self.buffer_index >= self.actual_buffer_length {
            self.buffer_start += self.actual_buffer_length;
            self.load_buffer()?;
        }

        if self.buffer_index < self.actual_buffer_length {
            let val = self.buffer[self.buffer_index];
            self.buffer_index += 1;
            Ok(val)
        }

        Err(io::Error::other("Invalid position! {core::io::read::BufferedReader<R>::get(&mut self);}"))
    }

    pub fn unget(&mut self) -> Result<(), io::Error>{
        if self.buffer_index == 0 {
            if self.buffer_start <= 0 {
                Err(io::Error::other("At start of stream cannot unget. {core::io::read::BufferedReader<R>::unget(&mut self);}"))
            }
            self.buffer_start -= std::cmp::min(self.buffer_start, BUFF_SIZE);
            self.load_buffer()?;
            return Ok(self.buffer_index = self.actual_buffer_length)
        } else if self.buffer_start > 0 {
            Ok(self.buffer_index -= 1)
        }

        panic!("Buffer index is less than 0; this is impossible! You broke something :(")
    }

}


