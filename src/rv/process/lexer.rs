use std::io;
use bex::{Analyser, Lexer, ScopedToken, Token};
use thiserror::Error;

type LexerResult<T> = Result<T, ProcError>;

#[derive(Error, Debug)]
pub enum ProcError {
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error("Unknown Directive Encountered")]
    UnknownDirective(Vec<u8>),
    #[error("Missing Space In Directive Body")]
    MissingSpace,
    #[error("Expected Identifier but found char outside of range.")]
    ExpectedId,
    #[error("Expected ., instead got .")]
    ExpectedText(),
    #[error("Expected ., instead got .")]
    UnexpectedChar(u8)
}

#[derive(PartialEq)]
pub struct ProcScope {
    pub line_number:    u32,
    pub quoted:         bool,
    pub new_line:       bool
}

#[derive(PartialEq)]
pub struct ProcMacroDefinition {
    pub macro_name:    Vec<u8>,
    pub macro_params:  Vec<Vec<u8>>
}

#[derive(PartialEq)]
pub struct ProcMacro {
    pub definition: ProcMacroDefinition,
    pub value:      Vec<u8>
}

#[derive(PartialEq)]
pub struct ProcIfBlock {
    pub negated:      bool,
    pub target:       Vec<u8>,
    pub if_section:   Vec<u8>,
    pub else_section: Option<Vec<u8>>
}

#[derive(PartialEq)]
pub struct ProcInclude {
    pub using_angles: bool,
    pub path:         Vec<u8>
}

#[derive(PartialEq)]
pub enum ProcToken {
    Include(ProcInclude), Define(ProcMacro), IfBlock(ProcIfBlock), Text(Vec<u8>),
    Undefine(Vec<u8>), Identifier(Vec<u8>), Unknown(Vec<u8>), DoubleHash,
    NewLine, LineBreak, Comment, LeftParenthesis, RightParenthesis, Comma
}

impl Default for ProcScope {
    fn default() -> Self {
        Self {
            line_number: 0,
            quoted: false,
            new_line: true
        }
    }
}

fn read_macro_name(
    lexer: &mut Lexer<u8>,
    scope: &mut ProcScope
) -> LexerResult<Vec<u8>> {
    let next = get_stripped(lexer, &mut scope.line_number)?;
    if !valid_id_char(next, true) {
        return Err(ProcError::ExpectedId)
    }
    Ok(get_name(lexer, &mut scope.line_number, Some(next), 128)?)
}

impl ScopedToken<u8> for ProcToken {
    type Scope = ProcScope;
    type Error = ProcError;

    fn next_token(
        lexer: &mut Lexer<u8>,
        scope: &mut ProcScope
    ) -> Result<Self, Self::Error> {
        let token = match scope.quoted {
            true => ProcToken::Text(std::iter::once(b'"').chain(lexer.get_until(b'"')?).chain(std::iter::once(b'"')).collect()),
            false => match get_stripped_not(lexer, &mut scope.line_number, b'\r')? {
                b'(' => ProcToken::LeftParenthesis,
                b')' => ProcToken::RightParenthesis,
                b',' => ProcToken::Comma,
                b'"' => { scope.quoted = true; <ProcToken as ScopedToken<u8>>::next_token(lexer, scope)? },
                b'\n' => { scope.line_number += 1; ProcToken::NewLine },
                b'#' => match get_stripped(lexer, &mut scope.line_number)? {
                    b'#' => ProcToken::DoubleHash,
                    next if scope.new_line && valid_id_char(next, true) => {
                        match get_name(lexer, &mut scope.line_number, Some(next), 128)?.as_slice() {
                            b"include" if skip_space(lexer)? => {
                                let terminator = match get_stripped(lexer, &mut scope.line_number)? {
                                    b'<' => b'>',
                                    current if current == b'"' => current,
                                    tok => return Err(ProcError::UnexpectedChar(tok))
                                };
                                let path=iterate_by_condition(lexer, &mut scope.line_number, 128, None, |current|
                                    current == terminator
                                )?;
                                ProcToken::Include(ProcInclude {
                                    using_angles: terminator == b'>',
                                    path
                                })
                            },
                            b"define" if skip_space(lexer)? => todo!(),
                            b"ifdef" => read_if(lexer, scope, false)?,
                            b"ifndef" => read_if(lexer, scope, true)?,
                            b"undef" =>  ProcToken::Undefine(read_macro_name(lexer, scope)?),
                            directive => ProcToken::Text({
                                let mut text = Vec::new();
                                text.push(b'#');
                                text.extend_from_slice(directive);
                                text
                            })
                        }
                    }
                    _ => { lexer.step_back()?; ProcToken::Text(vec![b'#']) }
                },
                current if valid_id_char(current, true) => ProcToken::Identifier(get_name(lexer, &mut scope.line_number, Some(current), 128)?),
                current if current == b'\\' || current == b'/' => {
                    let is_forward = current == b'/';
                    match get_stripped_not(lexer, &mut scope.line_number, b'\r')? {
                        b'/' if is_forward => {
                            lexer.seek_until(b'\n')?;
                            ProcToken::Comment
                        },
                        b'*' if is_forward => {
                            let mut current = lexer.get()?;
                            let mut last: u8 = 0;
                            while last != b'*' || current != b'/' {
                                last = current;
                                current = lexer.get()?;
                                if current == b'\n' {
                                    scope.line_number += 1;
                                }
                            }
                            ProcToken::Comment
                        },
                        b'\n' if !is_forward => {
                            scope.line_number += 1;
                            ProcToken::LineBreak
                        },
                        _ => {
                            lexer.step_back()?;
                            ProcToken::Unknown(vec![current])
                        }
                    }
                }
                unknown => ProcToken::Unknown(vec![unknown]),
            }
        };
        if scope.new_line && token != ProcToken::NewLine {
            scope.new_line = false;
        }
        Ok(token)
    }
}


fn skip_space(lexer: &mut Lexer<u8>) -> LexerResult<bool> {
    let mut found_space = false;
    loop {
        let current = *lexer.peek()?;
        if current < 33 && current != b'\n' {
            found_space = true;
            lexer.step_forward()?
        } else { break }
    }
    Ok(found_space)
}


fn read_if(
    lexer: &mut Lexer<u8>,
    scope: &mut ProcScope,
    negated: bool
) -> LexerResult<ProcToken> {
    if !skip_space(lexer)? {
        return Err(ProcError::MissingSpace)
    }
    todo!()
}


fn get_stripped(
    lexer: &mut Lexer<u8>,
    line_count: &mut u32
) -> LexerResult<u8> {
    let mut current = lexer.get_not(b'\r')?;
    while current == b'\\' {
        if lexer.get_not(b'\r')? != b'\n' {
            lexer.step_back()?;
            return Ok(b'\r')
        }
        *line_count += 1;
        current = lexer.get_not(b'\r')?
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
    mut check_condition: impl FnMut(u8) -> bool
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