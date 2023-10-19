use std::error::Error;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;
use crate::{Analyser, Lexer, LexicalError};

pub trait LexicalPreProcessor {
    type E: Error + From<LexicalError>;

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


pub trait Parseable: Sized where Self::P: LexicalPreProcessor<E = Self::E> {
    type E: Error + From<LexicalError> + From<io::Error>;
    type P: LexicalPreProcessor;

    fn parse_virtual_file<_P: AsRef<Path>>(path: _P, preprocessor: Option<&mut Self::P>) -> Result<Self, Self::E> {
        todo!("rv::vfs")
    }

    fn parse_system_file<_P: AsRef<Path>>(path: _P, preprocessor: Option<&mut Self::P>) -> Result<Self, Self::E> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        Self::parse_data(&buffer, preprocessor)
    }

    fn parse_data<B: AsRef<[u8]>>(content: B, preprocessor: Option<&mut Self::P>) -> Result<Self, Self::E> {
        let mut lexer = Lexer::new(content, true);
        return match preprocessor {
            None => Self::try_parse(&mut lexer),
            Some(proc) => Self::try_process_parse(lexer, proc)
        }
    }

    fn parse(mut lexer: Lexer) -> Self { Self::try_parse(&mut lexer).unwrap() }

    fn try_process_parse(mut lexer: Lexer, preprocessor: &mut Self::P) -> Result<Self, Self::E>{
        preprocessor.process_from_current_and_return(&mut lexer)?;
        Self::try_parse(&mut lexer)
    }

    fn try_parse(lexer: &mut Lexer) -> Result<Self, Self::E>;
}