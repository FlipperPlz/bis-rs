use std::error::Error;
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

impl Lexer {
    pub fn new<B: AsRef<[u8]>>(contents: B, mutable: bool) -> Self {
        Self {
            mutable,
            cursor: 0,
            contents: Vec::from(contents.as_ref()),
        }
    }
}
pub trait Tokenizer: Analyser<u8> {
    type Token: Sized;
    fn next_token(&mut self) -> Self::Token;
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

