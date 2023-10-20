use std::{io};
use log::error;
use thiserror::Error;
use crate::{Analyser, Lexer, ParamLiteral, ScopedTokenizer, Tokenizer};
use std::string::String;

#[derive(Error, Debug)]
pub enum ParamLexerError {
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error("EOF Reached")]
    EndReached,
    #[error("Missing expected token.")]
    ExpectedToken,
    #[error("Unknown string escape.")]
    UnknownEscape
}

impl Default for ParamLexicalScope { fn default() -> Self { Self::Statement } }
pub enum ParamLexicalScope {
    Statement,
    Expression
}

pub enum ParamToken {
    ClassKeyword, DeleteKeyword, EnumKeyword,
    SemiColon, Colon, Comma, Ampersand,
    LeftSquare, RightSquare, LeftCurly, RightCurly,
    Assign, AddAssign, SubAssign, LineDirective,
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
        match self.get().ok_or(ParamLexerError::EndReached)? {
            b';' => Ok(ParamToken::SemiColon),
            b':' => Ok(ParamToken::Colon),
            b'{' => Ok(ParamToken::LeftCurly),
            b'}' => Ok(ParamToken::RightCurly),
            b'[' => Ok(ParamToken::LeftSquare),
            b']' => Ok(ParamToken::RightSquare),
            b',' => Ok(ParamToken::Comma),
            b'=' => Ok(ParamToken::Assign),
            b'@' => Ok(ParamToken::Ampersand),
            b'-' => self.unknown_unless_next(b'=', ParamToken::SubAssign),
            b'+' => self.unknown_unless_next(b'=', ParamToken::AddAssign),
            b'#' => { self.process_directive()?; self.next_statement()},
            found => Ok(ParamToken::Unknown(vec![found]))
        }
    }

    fn process_directive(&mut self) -> LexerResult<()> {
        Ok(match self.take_word()? {
            b"line" => self.process_line_directive()?,
            _ => return Err(ParamLexerError::ExpectedToken)
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

    fn take_string(&mut self, terminators: &[u8]) -> LexerResult<ParamLiteral> {
        let mut current = self.skip_space()?;
        let mut contents: Vec<u8> = vec![];
        let quoted = match current {
            b'"' => {
                current = self.get().ok_or(ParamLexerError::EndReached)?;
                loop {
                    match current {
                        b'"' => if self.try_match_quoted_string_end(&mut current)? { break },
                        c if c == b'\n' ||c == b'\r' => return Err(ParamLexerError::ExpectedToken),
                        _ => {}
                    }
                    contents.push(current)
                }
                true
            },
            _ => {
                while !terminators.contains(&current) {
                    if current == b'\n' || current == b'\r' {
                        if self.try_match_terminated_string_end(&mut current, terminators)? { break }
                    }

                    contents.push(current)
                }
                while !contents.is_empty() && is_space(contents.last().unwrap()) { contents.pop(); }

                self.step_back().ok_or(ParamLexerError::EndReached)?;
                false
            }
        };
        return Ok(ParamLiteral::String(quoted, String::from_utf8(contents).unwrap()))
    }

    fn skip_space(&mut self) -> LexerResult<u8> {
        let mut current = self.peek().ok_or(ParamLexerError::EndReached)?;
        while is_space(&current) {
            current = self.get().ok_or(ParamLexerError::EndReached)?;
        }
        return Ok(current);
    }


    fn try_match_quoted_string_end(&mut self, current: &mut u8) -> LexerResult<bool> {
        *current = self.get().ok_or(ParamLexerError::EndReached)?;
        match current {
            b'"' => Ok(false),
            _ => {
                *current = self.skip_space()?;
                if !self.take(&b'\\') {
                    *current = self.step_back().ok_or(ParamLexerError::EndReached)?;
                    return Ok(true)
                }
                if !self.take(&b'n') {
                    return Err(ParamLexerError::UnknownEscape)
                }
                *current = self.skip_space()?;
                if !self.take(&b'"') {
                    return Err(ParamLexerError::UnknownEscape)
                }
                *current = b'\n';
                Ok(false)
            }
        }
    }

    fn try_match_terminated_string_end(&mut self, current: &mut u8, terminators: &[u8]) -> LexerResult<bool> {
        loop {
            *current = self.skip_space()?;
            if *current != b'#' {
                return Ok(false)
            }
            self.process_directive()?;
            *current = self.skip_space()?;

            if !terminators.contains(current) {
                return Err(ParamLexerError::ExpectedToken)
            }
            self.step_back();
            return Ok(true)
        }
    }
    fn process_line_directive(&self) -> LexerResult<()> {
        todo!()
    }
}

impl ScopedTokenizer for ParamLexer {
    type Token = ParamToken;
    type Error = ParamLexerError;
    type Scope = ParamLexicalScope;

    fn next_token(&mut self, scope: &Self::Scope) -> LexerResult<Self::Token> {
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
            ParamToken::Unknown(it) if it.len() > 0 && !is_numeric(it.first().unwrap())  => ParamToken::Identifier(it),
            _ => self,
        }
    }
}
