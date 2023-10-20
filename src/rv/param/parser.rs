use thiserror::Error;
use crate::{ParamLexer, ParamLexerError, ParamLexicalScope, Parser, ScopedTokenizer, Tokenizer};
use crate::param::ParamFile;

#[derive(Error, Debug)]
pub enum ParamParseError {
    #[error(transparent)]
    Lexical(#[from] ParamLexerError),


}

impl Parser for ParamFile {
    type E = ParamParseError;

    fn try_parse(lexer: &mut ParamLexer) -> Result<Self, Self::E> {
        let current_scope = ParamLexicalScope::Statement;
        let mut current_token = ScopedTokenizer::next_token(lexer, &current_scope)?;
        todo!()
    }
}

