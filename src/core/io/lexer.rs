
use thiserror::Error;
use crate::{Analyser, AnalysisError, MutAnalyser};

#[derive(Debug, Error)]
pub enum LexicalError {
    #[error(transparent)]
    Analysis(#[from] AnalysisError),
    #[error("This lexer is not mutable")]
    Immutable,
}

pub struct Lexer {
    mutable:     bool,
    cursor:      usize,
    contents:    Vec<u8>
}

impl Analyser<u8> for Lexer {
    type E = LexicalError;

    #[inline]
    fn contents(&self) -> &Vec<u8> { &self.contents }

    #[inline]
    fn pos(&self) -> usize { self.cursor }

    #[inline]
    fn set_cursor(&mut self, cursor: usize) { self.cursor = cursor; }
}

impl MutAnalyser<u8> for Lexer {
    fn contents_mut(&mut self) -> Result<&mut Vec<u8>, Self::E> {
        return if !self.mutable { Err(LexicalError::Immutable) } else {
            Ok(&mut self.contents)
        }
    }
}

