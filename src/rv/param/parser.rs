use thiserror::Error;
use crate::{ParamLexer, ParamLexerError, Parser, Tokenizer};
use crate::param::ParamFile;

#[derive(Error, Debug)]
pub enum ParamParseError {
    #[error(transparent)]
    Lexical(#[from] ParamLexerError),


}

impl Parser for ParamFile {
    type E = ParamParseError;

    fn try_parse(lexer: &mut ParamLexer) -> Result<Self, Self::E> {
        let mut current_token = lexer.next_token()?;
        todo!()
    }
}

