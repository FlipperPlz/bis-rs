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

    fn parse_data<B: AsRef<[u8]>>(content: B, preprocessor: Option<&mut dyn LexicalPreProcessor<E=Self::E>>) -> Result<Self, Self::E> {
        let mut lexer = Lexer::new(content, true);
        return match preprocessor {
            None => Self::try_parse(&mut lexer),
            Some(proc) => Self::try_process_parse(lexer, proc)
        }
    }

    fn parse(mut lexer: Lexer) -> Self { Self::try_parse(&mut lexer).unwrap() }

    fn try_process_parse(mut lexer: Lexer, preprocessor: &mut dyn LexicalPreProcessor<E=Self::E>) -> Result<Self, Self::E>{
        preprocessor.process_from_current_and_return(&mut lexer)?;
        Self::try_parse(&mut lexer)
    }

    fn try_parse(lexer: &mut Lexer) -> Result<Self, Self::E>;
}