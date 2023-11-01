use std::io;
use bex::{Lexer, PreProcess};
use thiserror::Error;
use crate::{lexer, ProcInclude, ProcMacro, ProcToken};
type PreprocessorResult<T> = Result<T, PreprocessorError>;

pub struct RvPreprocessor {

}
#[derive(Error, Debug)]
pub enum PreprocessorError {
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error(transparent)]
    Lexer(#[from] lexer::ProcError)
}

impl RvPreprocessor {
    fn try_expand_macro(
        &mut self,
        macro_name:  &Vec<u8>,
        token_stack: &mut Vec<ProcToken>
    ) -> PreprocessorResult<Option<Vec<u8>>> {
        todo!()
    }

    fn undefine(
        &mut self,
        macro_name: &Vec<u8>
    ) -> PreprocessorResult<Option<ProcMacro>> {
        todo!()
    }

    fn defined(
        &mut self,
        macro_name: &Vec<u8>
    ) -> PreprocessorResult<bool> {
        todo!()
    }

    fn define(
        &mut self,
        mac: ProcMacro
    ) -> PreprocessorResult<()> {
        todo!()
    }

    fn process_include(&mut self, include: ProcInclude) -> PreprocessorResult<Vec<u8>> {
        self.preprocess_file(include.path)
    }

    fn preprocess_file(&mut self, path: Vec<u8>) -> PreprocessorResult<Vec<u8>> {
        let file_contents: Vec<u8> = todo!();
        return PreProcess::preprocess(self, Lexer::new(file_contents))
    }
}
impl PreProcess<u8> for RvPreprocessor {
    type E = PreprocessorError;

    fn preprocess(&mut self, lexer: Lexer<u8>) -> Result<Vec<u8>, Self::E> {
        let mut document: Vec<u8> = vec![];
        let mut contents = lexer.tokenize_until_end::<ProcToken>()?;
        contents.reverse();
        while !contents.is_empty() {
            match match contents.pop() {
                None => break,
                Some(it) => it
            } {
                ProcToken::Include(include) => self.process_include(include),
                ProcToken::Define(it) => self.define(it),
                ProcToken::IfBlock(block) => if block.negated ^ self.defined(&block.target) {
                    document.extend(block.if_section)
                } else if let Some(text) = block.else_section {
                    document.extend(text)
                },
                ProcToken::Undefine(it) => self.undefine(&it)?,
                ProcToken::Identifier(id) => match self.try_expand_macro(&id, &mut contents)? {
                    None => { document.extend(id); }
                    Some(expanded) => document.extend(expanded)
                }
                ProcToken::Unknown(text) => document.extend(text),
                ProcToken::DoubleHash => match contents.last() {
                    None => document.extend_from_slice(&[b'#', b'#']),
                    Some(tok) => match tok {
                        ProcToken::Identifier(text) => {
                            match self.try_expand_macro(text, &mut contents)? {
                                None => {
                                    document.extend_from_slice(&[b'#', b'#']);
                                    document.extend(text);
                                }
                                Some(expanded) => document.extend(expanded)
                            }
                        },
                        _ => {document.extend_from_slice(&[b'#', b'#'])}
                    }
                }
                ProcToken::NewLine => document.push(b'\n'),
                ProcToken::LineBreak => {},
                ProcToken::Comment => {},
                ProcToken::Text(text) => document.extend(text),
                ProcToken::LeftParenthesis => document.push(b'('),
                ProcToken::RightParenthesis => document.push(b')'),
                ProcToken::Comma => document.push(b','),
            }
        }

        return Ok(document)
    }
}