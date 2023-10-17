use std::io;
use std::io::{Read, Seek};
use std::str::FromStr;
use crate::{BufferedReader, PredicateOption};
pub const CONST_DOUBLE_QUOTE: &str = "\"";
pub const CONST_RIGHT_ANGLE: &str = ">";

const CONST_TOKENS: [(LexToken, &str); 19] = [
    (LexToken::Include, "include"),
    (LexToken::Define, "define"),
    (LexToken::IfDef, "ifdef"),
    (LexToken::IfNDef, "ifndef"),
    (LexToken::Else, "else"),
    (LexToken::EndIf, "endif"),
    (LexToken::LeftParenthesis, "("),
    (LexToken::RightParenthesis, ")"),
    (LexToken::Comma, ","),
    (LexToken::Hash, "#"),
    (LexToken::NewLine, "\n"),
    (LexToken::LineCommentStart, "//"),
    (LexToken::DelimitedCommentStart, "/*"),
    (LexToken::LineBreak, "\\\n"),
    (LexToken::DQuote, CONST_DOUBLE_QUOTE),
    (LexToken::LeftAngle, "<"),
    (LexToken::RightAngle, CONST_RIGHT_ANGLE),
    (LexToken::DoubleHash, "##"),
    (LexToken::Undef, "undef")
];
#[derive(Debug, Clone)]
pub enum LexToken {
    Include,
    Define,
    IfDef,
    IfNDef,
    Else,
    EndIf,
    LeftParenthesis,
    RightParenthesis,
    Comma,
    Hash,
    NewFile,
    NewLine,
    LineCommentStart,
    DelimitedCommentStart,
    LineBreak,
    DQuote,
    LeftAngle,
    RightAngle,
    DoubleHash,
    Undef,
    Text(String),
    Unknown(String)
}

fn const_token(string: &String) -> LexToken {
    for (item, text) in CONST_TOKENS {
        if string.starts_with(text) {
            return item;
        }
    }
    return LexToken::Unknown(string.clone())
}


pub struct PreprocessorReader<R: Read + Seek> {
    reader:              BufferedReader<R>,
    directive_newlines:  u32
}

impl<R: Read + Seek> PreprocessorReader<R> {
    
    pub fn new(reader: R) -> Self {
        Self {
            reader: BufferedReader::new(reader),
            directive_newlines: 0,
        }
    }

    #[inline(always)]
    fn unget(&mut self) -> Result<(), io::Error> { self.reader.unget() }

    #[inline(always)]
    fn get(&mut self) -> Result<u8, io::Error> { self.reader.get() }


    #[inline(always)]
    pub fn directive_newline_count(&self) -> u32 { self.directive_newlines }


    #[inline(always)]
    pub fn pop_newline_count(&mut self) { self.directive_newlines -= 1 }


    #[inline(always)]
    pub fn reset_newline_count(&mut self) { self.directive_newlines = 0 }

    #[inline(always)]
    fn get_not(&mut self, stripped: bool, not: u8) -> Result<u8, io::Error> {
        if stripped {
            let mut current = self.get_stripped()?;
            while current == not { current = self.get_stripped()? }
            Ok(current)
        } else { self.reader.get_not(not) }
    }


    pub fn next_token(&mut self, mut token_text: &mut String, max_length: usize) -> Result<LexToken, io::Error> {
        let mut current = self.get_not(true, b'\r')?; self.unget()?;
        if let Some(mut token) = self.scan_name(max_length)? {
            *token_text = token;
            return Ok(match const_token(token_text) {
                LexToken::Unknown(s) => LexToken::Text(s),
                other => other,
            });
        } else { current = self.get()?; }
        token_text.clear();

        if current == b'\\' || current == b'/' {
            token_text.push(current as char);
            current = self.get_not(true, b'\r')?;
            if current == b'\n' || current == b'*' || current == b'/' {
                token_text.push(current as char);
                token_text.pop();
            } else { self.unget()?; }
        } else if current == b'#' {
            current = self.get_stripped()?;
            if current != b'#' { self.unget()?; } else { token_text.push(current as char) }
        }
        Ok(const_token(token_text))
    }

    pub fn scan_name(&mut self, max_length: usize) -> Result<Option<String>, io::Error> {
        let mut first = true;
        let mut size = 0;

        Ok(self.next_while(true, |next| {
            if size < max_length { return PredicateOption::Exit }
            size += 1;
            let state: PredicateOption = match Self::valid_identifier_char(next, first) {
                true => PredicateOption::Continue,
                false => PredicateOption::Exit,
            };
            if first { first = false; }
            state
        })?)
    }

    pub fn scan_string(&mut self, max_length: usize, terminators: &str) -> Result<Option<String>, io::Error> {
        let mut size = 0;
        Ok(self.next_while(false, |next| {
            size += 1;
            if size < max_length || terminators.as_bytes().contains(next) { PredicateOption::Exit }
            else {PredicateOption::Continue}
        })?)
    }

    pub fn skip_whitespace(&mut self) -> Result<u8, io::Error>{
        loop {
            let i = self.get()?;
            if i < 33 && i != b'\n' {
                return Ok(i)
            }
        }
    }

    fn get_stripped(&mut self) -> Result<u8, io::Error>{
        let mut current = self.get_not(false, b'\r')?;
        while current == b'\\' {
            if self.get_not(false, b'\r')? != b'\n' {
                self.unget()?;
                return Ok(current)
            }
            current = self.get()?;
            self.directive_newlines += 1;
            current = self.get_not(false, b'\r')?
        }

        Ok(current)
    }

    fn valid_identifier_char(char: &u8, is_first: bool) -> bool {
        match char {
            b'0'..=b'9' => !is_first,
            b'a'..=b'z' => true,
            _ => false
        }
    }

    fn next_while(&mut self, use_stripped: bool, mut predicate: impl FnMut(&mut u8) -> PredicateOption) -> Result<Option<String>, io::Error> {
        let mut string = String::new();
        loop {
            let mut peeked = if use_stripped { self.get_stripped()? } else { self.get()? };
            match predicate(&mut peeked) {
                PredicateOption::Skip => { continue }
                PredicateOption::Continue => {string.push(peeked as char); continue}
                PredicateOption::Exit => { self.unget()?; return Ok(if string.is_empty() { None } else { Some(string) })}
                PredicateOption::Err(e) => { self.unget()?; return Err(e) }
            }
        }
    }

}
