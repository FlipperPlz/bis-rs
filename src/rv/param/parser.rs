use std::collections::HashMap;
use thiserror::Error;
use crate::{ParamConstant, ParamExpression, ParamLexer, ParamLexerError, ParamLexicalScope, ParamStatement, ParamToken, Parser, ScopedTokenizer, Tokenizer};
use crate::param::{ConstantId, ContextId, ExpressionId, ParamFile, StatementGroup, StatementId};

#[derive(Error, Debug)]
pub enum ParamParseError {
    #[error(transparent)]
    Lexical(#[from] ParamLexerError),


}

impl Parser for ParamFile {
    type E = ParamParseError;

    fn try_parse(filename: String, lexer: &mut ParamLexer) -> Result<Self, Self::E> {

        match ScopedTokenizer::next_token(lexer, &ParamLexicalScope::Expression)? {
            ParamToken::ClassKeyword => {}
            ParamToken::DeleteKeyword => {}
            ParamToken::EnumKeyword => {}
            ParamToken::SemiColon => {}
            ParamToken::Colon => {}
            ParamToken::Comma => {}
            ParamToken::Ampersand => {}
            ParamToken::LeftSquare => {}
            ParamToken::RightSquare => {}
            ParamToken::LeftCurly => {}
            ParamToken::RightCurly => {}
            ParamToken::Assign => {}
            ParamToken::AddAssign => {}
            ParamToken::SubAssign => {}
            ParamToken::LineDirective => {}
            ParamToken::Identifier(_) => {}
            ParamToken::LiteralString { .. } => {}
            ParamToken::Integer(_) => {}
            ParamToken::Float(_) => {}
            ParamToken::Double(_) => {}
            ParamToken::Unknown(_) => {}
        }
        todo!()
    }
}

