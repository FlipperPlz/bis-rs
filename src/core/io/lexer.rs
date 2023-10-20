use std::error::Error;
use std::io;
use crate::{Analyser, MutAnalyser};

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

pub trait Tokenizer {
    type Token;
    type Error: Error;

    fn next_token(&mut self) -> Result<Self::Token, Self::Error>;
}

pub trait ScopedTokenizer {
    type Token;
    type Error: Error;
    type Scope;

    fn next_token(&mut self, scope: &Self::Scope) -> Result<Self::Token, Self::Error>;
}

impl<Tok: ScopedTokenizer<Scope = S>, S: Default> Tokenizer for Tok {
    type Token = Tok::Token;
    type Error = Tok::Error;

    fn next_token(&mut self) -> Result<Self::Token, Self::Error> {
        self.next_token(&S::default())
    }
}

impl Analyser<u8> for Lexer {

    #[inline]
    fn contents(&self) -> &Vec<u8> { &self.contents }

    #[inline]
    fn pos(&self) -> usize { self.cursor }

    #[inline]
    fn set_cursor(&mut self, cursor: usize) { self.cursor = cursor; }
}

impl MutAnalyser<u8> for Lexer {
    fn contents_mut(&mut self) -> io::Result<&mut Vec<u8>> {
        return if !self.mutable { Err(io::Error::other("Lexer is immutable.")) } else {
            Ok(&mut self.contents)
        }
    }
}

