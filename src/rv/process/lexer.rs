use std::io;
use bex::lexer::{Lexer, ScopedToken, Token};
use bex::read::Analyser;
use thiserror::Error;

type LexerResult<T> = Result<T, ProcLexicalError>;

#[derive(Error, Debug)]
pub enum ProcLexicalError {
    #[error(transparent)]
    IO(#[from] io::Error),
}

pub struct ProcLexicalScope {
    line_number: u32,
    defines:     Vec<ProcMacro>,
    quoted:      bool
}

pub struct ProcMacroDefinition {
    macro_name: Vec<u8>,
    macro_params: Vec<Vec<u8>>
}

pub struct ProcMacro {
    definition: ProcMacroDefinition,
    value:      Vec<u8>
}

pub struct ProcIfBlock {
    target:  Vec<u8>,
    if_section: Vec<u8>,
    else_section: Option<Vec<u8>>
}

pub enum ProcLexicalToken {
    Include(Vec<u8>), Define(ProcMacro),
    IfDef(ProcIfBlock), IfNDef(ProcIfBlock),
    Identifier(Vec<u8>), Text(Vec<u8>), Unknown(Vec<u8>),
    LineBreak, Comment
}


impl ScopedToken<u8> for ProcLexicalToken {
    type Scope = ProcLexicalScope;
    type Error = ProcLexicalError;

    fn next_token(
        lexer: &mut Lexer<u8>,
        scope: &mut Self::Scope
    ) -> Result<Self, Self::Error> {
        if scope.quoted {
            let text = ProcLexicalToken::Text(std::iter::once(b'"').chain(lexer.get_until(b'\"').into_iter()).chain(std::iter::once(b'"')).collect());
            lexer.step_forward()?;
            return Ok(text)
        }

        let mut current = get_stripped_not(lexer, &mut scope.line_number, 0x0d)?;
        if valid_id_char(current, true) {
            return Ok(ProcLexicalToken::Identifier(get_name(lexer, &mut scope.line_number, Some(current), 128)?))
        }

        return match current {
            b'#' if !scope.quoted => {
                if lexer.take(&b'#') { read_macro(lexer, scope) }
                else { next_directive(lexer, scope) }
            }
            b'"' => {
                scope.quoted = true;
                Self::next_token(lexer, scope);
            }
            b'/' if !scope.quoted => {
                if lexer.take(&b'/') { read_line_comment(lexer, scope) }
                else if lexer.take(&b'*') { read_delimited_comment(lexer, scope)}
                else { Ok(ProcLexicalToken::Unknown(Vec::from(b'/'))) }
            }
            content => Ok(ProcLexicalToken::Unknown(Vec::from(content)))
        }
    }
}

fn read_delimited_comment(
    lexer: &mut Lexer<u8>,
    scope: &mut ProcLexicalScope
) -> LexerResult<ProcLexicalToken> {
    let mut current = lexer.get()?;
    let mut last: u8 = 0;
    while last != b'*' || current != b'/' {
        last = current;
        current = lexer.get()?;
        if current == b'\n' {
            scope.line_number += 1;
        }
    }

    return Ok(ProcLexicalToken::Comment)
}

fn read_line_comment(
    lexer: &mut Lexer<u8>,
    scope: &mut ProcLexicalScope
) -> LexerResult<ProcLexicalToken> {
    let mut current = lexer.get()?;
    while current != b'\n' {
        current = lexer.get()?;
    }
    scope.line_number += 1;

    return Ok(ProcLexicalToken::Comment)
}

fn read_macro(
    lexer: &mut Lexer<u8>,
    scope: &mut ProcLexicalScope
) -> LexerResult<ProcLexicalToken> {
    todo!()
}

fn next_directive(
    lexer: &mut Lexer<u8>,
    scope: &mut ProcLexicalScope
) -> LexerResult<ProcLexicalToken> {
    todo!()

}

fn get_stripped(
    lexer: &mut Lexer<u8>,
    line_count: &mut u32
) -> LexerResult<u8> {
    let mut current = lexer.get_not(b'\r')?;
    while current == b'\\' {
        if *lexer.get_not(b'\r')? != b'\n' {
            lexer.step_back()?;
            return Ok(b'\r')
        }
        *line_count += 1;
        current = *lexer.get_not(b'\r')?
    }
    Ok(current)
}

fn get_stripped_not(
    lexer: &mut Lexer<u8>,
    line_count: &mut u32,
    target: u8,
) -> LexerResult<u8> {
    let mut found: u8;
    loop {
        found = get_stripped(lexer, line_count)?;
        if found == target { continue }
        return Ok(found);
    }
}

fn iterate_by_condition(
    lexer: &mut Lexer<u8>,
    line_count: &mut u32,
    max_size: u32,
    use_first: Option<u8>,
    check_condition: impl Fn(u8) -> bool
) -> LexerResult<Vec<u8>> {
    let mut max_size = max_size;
    let mut buffer: Vec<u8> = vec![];
    if max_size == 0 { return Ok(buffer) }
    let mut current = match use_first {
        None => get_stripped(lexer, line_count)?,
        Some(first) => first
    };
    while check_condition(current) {
        buffer.push(current);
        max_size -= 1;
        if max_size == 0 { return Ok(buffer) }
        current = get_stripped(lexer, line_count)?;
    }
    lexer.step_back()?;
    Ok(buffer)
}

fn get_string(
    lexer: &mut Lexer<u8>,
    line_count: &mut u32,
    use_first: Option<u8>,
    max_size: u32,
    terminators: &[u8],
) -> LexerResult<Vec<u8>> {
    iterate_by_condition(lexer, line_count, max_size, use_first, |current|
        !terminators.contains(&current)
    )
}

fn get_name(
    lexer: &mut Lexer<u8>,
    line_count: &mut u32,
    use_first: Option<u8>,
    max_size: u32
) -> LexerResult<Vec<u8>> {
    let mut is_first_char = true;
    iterate_by_condition(lexer, line_count, max_size, use_first, |current| {
        let is_valid = valid_id_char(current, is_first_char);
        is_first_char = false;
        is_valid
    })
}

pub fn valid_id_char(char: u8, is_first: bool) -> bool {
    (!is_first && (
        char >= b'0' ||
            char <= b'9')
    ) ||
        char >= b'a' ||
        char <= b'z' ||
        char >= b'A' ||
        char <= b'Z' ||
        char == b'_'
}

