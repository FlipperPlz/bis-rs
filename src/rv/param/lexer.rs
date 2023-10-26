use std::{io};
use bex::lexer::{Lexer, ScopedToken};
use bex::read::Analyser;
use log::error;
use thiserror::Error;

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
type LexerResult<T> = Result<T, ParamLexerError>;

impl Default for ParamLexicalScope { fn default() -> Self { Self::Statement } }

pub enum ParamLexicalScope {
    Statement,
    Expression,
    ArrayExpression
}

pub enum ParamToken {
    ClassKeyword, DeleteKeyword, EnumKeyword,
    SemiColon, Colon, Comma, Ampersand, EOF,
    LeftSquare, RightSquare, LeftCurly, RightCurly,
    Assign, AddAssign, SubAssign, LineDirective,
    Identifier(Vec<u8>), ExecuteDirective(Vec<u8>),
    LiteralString { double_quoted: bool, data: Vec<u8> },
    Integer(i32), Float(f32), Double(f64), Unknown(Vec<u8>)
}

impl ScopedToken<u8> for ParamToken {
    type Scope = ParamLexicalScope;
    type Error = ParamLexerError;

    fn next_token(lexer: &mut Lexer<u8>, scope: &Self::Scope) -> Result<Self, Self::Error> {
        match scope {
            ParamLexicalScope::Statement => next_statement(lexer),
            ParamLexicalScope::Expression => next_expression(lexer, false),
            ParamLexicalScope::ArrayExpression => next_expression(lexer, true),
        }
    }
}

fn next_statement(lexer: &mut Lexer<u8>) -> LexerResult<ParamToken> {
    return match take_word(lexer)? {
        word if word.is_empty() => next_symbol(lexer),
        word => Ok(match word {
            b"class" =>  ParamToken::ClassKeyword,
            b"enum" => ParamToken::EnumKeyword,
            b"delete" => ParamToken::DeleteKeyword,
            b"__EXEC" => ParamToken::ExecuteDirective(read_execute(lexer)?),
            unknown=> ParamToken::Unknown(Vec::from(unknown))
        }.or_else_identifier())
    };
}

fn next_symbol(lexer: &mut Lexer<u8>) -> LexerResult<ParamToken> {
    match lexer.get()? {
        b';' => Ok(ParamToken::SemiColon),
        b':' => Ok(ParamToken::Colon),
        b'{' => Ok(ParamToken::LeftCurly),
        b'}' => Ok(ParamToken::RightCurly),
        b'[' => Ok(ParamToken::LeftSquare),
        b']' => Ok(ParamToken::RightSquare),
        b',' => Ok(ParamToken::Comma),
        b'=' => Ok(ParamToken::Assign),
        b'@' => Ok(ParamToken::Ampersand),
        b'-' => unknown_unless_next(lexer, b'=', ParamToken::SubAssign),
        b'+' => unknown_unless_next(lexer, b'=', ParamToken::AddAssign),
        b'#' => { process_directive(lexer)?; next_statement(lexer)},
        found => Ok(ParamToken::Unknown(vec![found]))
    }
}

pub fn take_word(lexer: &mut Lexer<u8>) -> LexerResult<&[u8]> {
    let mut current = skip_space(lexer)?;
    let start = lexer.pos().clone();
    while current == b'_' || is_alphanumeric(&current) {
        current = lexer.get()?;
    }
    return Ok(&lexer.contents()[start..*lexer.pos()]);
}

#[inline]
fn unknown_unless_next(lexer: &mut Lexer<u8>, next: u8, correct: ParamToken) -> LexerResult<ParamToken> {
    Ok(if lexer.take(&next)? {
        correct
    } else {
        ParamToken::Unknown(vec![lexer.get()?])
    })
}

fn read_execute(lexer: &mut Lexer<u8>) -> LexerResult<Vec<u8>> {
    todo!()
}

fn next_expression(lexer: &mut Lexer<u8>, in_array: bool) -> LexerResult<ParamToken> {
    todo!()
}

fn process_line_directive(lexer: &mut Lexer<u8>) -> LexerResult<()> {
    todo!()
}

fn process_directive(lexer: &mut Lexer<u8>) -> LexerResult<()> {
    Ok(match take_word(lexer)? {
        b"line" => process_line_directive(lexer)?,
        _ => return Err(ParamLexerError::ExpectedToken)
    })
}

pub fn take_string(lexer: &mut Lexer<u8>, terminators: &[u8]) -> LexerResult<ParamToken> {
    let mut current = skip_space(lexer)?;
    let mut data: Vec<u8> = vec![];
    let double_quoted = match current {
        b'"' => {
            current = lexer.get()?;
            loop {
                match current {
                    b'"' => if try_match_quoted_string_end(lexer, &mut current)? { break },
                    c if c == b'\n' ||c == b'\r' => return Err(ParamLexerError::ExpectedToken),
                    _ => {}
                }
                data.push(current)
            }
            true
        },
        _ => {
            while !terminators.contains(&current) {
                if current == b'\n' || current == b'\r' {
                    if try_match_terminated_string_end(lexer, &mut current, terminators)? { break }
                }

                data.push(current)
            }
            while !data.is_empty() && is_space(data.last().unwrap()) { data.pop(); }

            lexer.step_back()?;
            false
        }
    };
    return Ok(ParamToken::LiteralString {
        double_quoted,
        data,
    })
}
pub fn skip_space(lexer: &mut Lexer<u8>) -> LexerResult<u8> {
    let mut current = *lexer.peek()?;
    while is_space(&current) {
        current = lexer.get()?;
    }
    return Ok(current);
}


fn try_match_quoted_string_end(lexer: &mut Lexer<u8>, current: &mut u8) -> LexerResult<bool> {
    *current = lexer.get()?;
    match current {
        b'"' => Ok(false),
        _ => {
            *current = skip_space(lexer)?;
            if !lexer.take(&b'\\')? {
                lexer.step_back()?;
                *current = *lexer.peek()?;
                return Ok(true)
            }
            if !lexer.take(&b'n')? {
                return Err(ParamLexerError::UnknownEscape)
            }
            *current = skip_space(lexer)?;
            if !lexer.take(&b'"')? {
                return Err(ParamLexerError::UnknownEscape)
            }
            *current = b'\n';
            Ok(false)
        }
    }
}

fn try_match_terminated_string_end(lexer: &mut Lexer<u8>, current: &mut u8, terminators: &[u8]) -> LexerResult<bool> {
    loop {
        *current = skip_space(lexer)?;
        if *current != b'#' {
            return Ok(false)
        }
        process_directive(lexer)?;
        *current = skip_space(lexer)?;

        if !terminators.contains(current) {
            return Err(ParamLexerError::ExpectedToken)
        }
        lexer.step_back()?;
        return Ok(true)
    }
}

fn is_space(c: &u8) -> bool { matches!(c, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c) }

fn is_alphanumeric(c: &u8) -> bool { is_alpha(c) || is_numeric(c) }

fn is_numeric(c: &u8) -> bool { matches!(c, b'0'..=b'9') }

fn is_alpha(c: &u8) -> bool { matches!(c, b'a'..=b'z' | b'A'..=b'Z') }

impl ParamToken {

    pub fn identifier_or_err<F: FnOnce(&ParamToken) -> F>(&self, failure: F) -> Result<&Vec<u8>, F> {
        match self {
            ParamToken::Identifier(it) => Ok(it),
            t => Err(failure(t))
        }
    }

    pub fn or_else_identifier(self) -> Self {
        match self {
            ParamToken::Unknown(it) if it.len() > 0 && !is_numeric(it.first().unwrap())  => ParamToken::Identifier(it),
            _ => self,
        }
    }
}
