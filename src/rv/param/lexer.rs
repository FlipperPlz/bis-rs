use std::{io};
use thiserror::Error;
use crate::{Analyser, Lexer, ScopedTokenizer, Tokenizer};


#[derive(Error, Debug)]
pub enum ParamLexerError {
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error("EOF Reached")]
    EndReached,
    #[error("Missing expected token.")]
    ExpectedToken
}

impl Default for ParamLexicalScope { fn default() -> Self { Self::Statement } }
pub enum ParamLexicalScope {
    Statement,
    Expression
}

pub enum ParamToken {
    ClassKeyword, DeleteKeyword, EnumKeyword,
    SemiColon, Colon, Comma,
    LeftSquare, RightSquare, LeftCurly, RightCurly,
    Assign, AddAssign, SubAssign,
    Identifier(Vec<u8>),
    LiteralString { double_quoted: bool, data: Vec<u8> },
    Integer(i32), Float(f32), Double(f64), Unknown(Vec<u8>)
}


type LexerResult<T> = Result<T, ParamLexerError>;
pub type ParamLexer = Lexer;

impl ParamLexer {
    fn next_statement(&mut self) -> LexerResult<ParamToken> {
        return match self.take_word()? {
            word if word.is_empty() => self.next_symbol(),
            word => Ok(Self::next_keyword(word).or_else_identifier())
        };
    }

    fn next_expression(&mut self) -> LexerResult<ParamToken> {
        todo!()
    }

    fn next_symbol(&mut self) -> LexerResult<ParamToken> {
        Ok(match self.get().ok_or(ParamLexerError::EndReached)? {
            b';' => ParamToken::SemiColon,
            b':' => ParamToken::Colon,
            b'{' => ParamToken::LeftCurly,
            b'}' => ParamToken::RightCurly,
            b'[' => ParamToken::LeftSquare,
            b']' => ParamToken::RightSquare,
            b',' => ParamToken::Comma,
            b'=' => ParamToken::Assign,
            b'-' => self.unknown_unless_next(b'=', ParamToken::SubAssign)?,
            b'+' => self.unknown_unless_next(b'=', ParamToken::AddAssign)?,
            found => ParamToken::Unknown(vec![found])
        })
    }

    fn next_keyword(found_word: &[u8]) -> ParamToken {
        match found_word {
            b"class" =>  ParamToken::ClassKeyword,
            b"enum" => ParamToken::EnumKeyword,
            b"delete" => ParamToken::DeleteKeyword,
            unknown=> ParamToken::Unknown(Vec::from(unknown))
        }
    }

    fn unknown_unless_next(&mut self, next: u8, correct: ParamToken) -> LexerResult<ParamToken> {
        Ok(if self.take(&next) {
            correct
        } else {
            ParamToken::Unknown(vec![self.get().ok_or(ParamLexerError::EndReached)?])
        })
    }

    fn take_word(&mut self) -> LexerResult<&[u8]> {
        let mut current = self.skip_space()?;
        let start = self.pos();
        while current == b'_' || is_alphanumeric(&current) {
            current = self.get().ok_or(ParamLexerError::EndReached)?;
        }
        return Ok(self.get_range(start, self.pos()));
    }

    fn skip_space(&mut self) -> LexerResult<u8> {
        let mut current = self.peek().ok_or(ParamLexerError::EndReached)?;
        while is_space(&current) && self.is_end() {
            current = self.get().ok_or(ParamLexerError::EndReached)?;
        }
        return Ok(current);
    }

}

impl ScopedTokenizer for ParamLexer {
    type Token = ParamToken;
    type Error = ParamLexerError;
    type Scope = ParamLexicalScope;

    fn next_token(&mut self, scope: Self::Scope) -> LexerResult<Self::Token> {
        return match scope {
            ParamLexicalScope::Statement => self.next_statement(),
            ParamLexicalScope::Expression => self.next_expression(),
        }
    }
}

fn is_space(c: &u8) -> bool { matches!(c, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c) }
fn is_alphanumeric(c: &u8) -> bool { is_alpha(c) || is_numeric(c) }
fn is_numeric(c: &u8) -> bool { matches!(c, b'0'..=b'9') }
fn is_alpha(c: &u8) -> bool { matches!(c, b'a'..=b'z' | b'A'..=b'Z') }

impl ParamToken {
    pub fn or_else_identifier(self) -> Self {
        match self {
            ParamToken::Unknown(it) if !is_numeric(it.first().unwrap())  => ParamToken::Identifier(it),
            _ => self,
        }
    }
}
