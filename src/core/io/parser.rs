use std::error::Error;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;
use crate::{Analyser, Lexer};

pub trait LexicalPreProcessor {
    type E: Error;

    fn process_from_start_and_return(&mut self, lexer: &mut Lexer) -> Result<(), Self::E>{
        let previous = lexer.reset();
        self.process(lexer)?;
        Ok(lexer.set_cursor(previous))
    }

    fn process_from_current_and_return(&mut self, lexer: &mut Lexer) -> Result<(), Self::E>{
        let previous = lexer.pos();
        self.process(lexer)?;
        Ok(lexer.set_cursor(previous))
    }

    fn process(&mut self, lexer: &mut Lexer) -> Result<(), Self::E>;
}


pub trait Parser: Sized {
    type E: Error;

    fn parse_virtual_file<P: AsRef<Path>>(path: P, preprocessor: Option<&mut dyn LexicalPreProcessor<E=Self::E>>) -> Result<Self, Self::E> {
        todo!("rv::vfs")
    }

    fn parse_data<B: AsRef<[u8]>>(filename: String, content: B, preprocessor: Option<&mut dyn LexicalPreProcessor<E=Self::E>>) -> Result<Self, Self::E> {
        let mut lexer = Lexer::new(content, true);
        return match preprocessor {
            None => Self::try_parse(filename, &mut lexer),
            Some(proc) => Self::try_process_parse(filename, lexer, proc)
        }
    }

    fn parse(filename: String, mut lexer: Lexer) -> Self { Self::try_parse(filename, &mut lexer).unwrap() }

    fn try_process_parse(filename: String, mut lexer: Lexer, preprocessor: &mut dyn LexicalPreProcessor<E=Self::E>) -> Result<Self, Self::E>{
        preprocessor.process_from_current_and_return(&mut lexer)?;
        Self::try_parse(filename, &mut lexer)
    }

    fn try_parse(filename: String, lexer: &mut Lexer) -> Result<Self, Self::E>;
}