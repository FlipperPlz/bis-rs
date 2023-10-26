use std::io;
use bex::lexer::{Lexer, Token};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParamLexerError {
    #[error(transparent)]
    IO(#[from] io::Error),
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

pub enum ProcToken {
    Include(Vec<u8>),
    MacroEvaluation(ProcMacroDefinition),
    Define(ProcMacro),
    IfDef(ProcIfBlock),
    IfNDef(ProcIfBlock),
    LineComment(Vec<u8>),
    DelimitedComment(Vec<u8>),
    Text(Vec<u8>),
    LineBreak,
    Unknown
}

impl Token<u8> for ProcToken {
    type Error = ParamLexerError;

    fn next_token(lexer: &mut Lexer<u8>) -> Result<Self, Self::Error> {
        todo!()
    }
}