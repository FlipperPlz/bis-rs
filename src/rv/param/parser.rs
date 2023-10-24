use std::str::Utf8Error;
use thiserror::Error;
use crate::{ParamClass, ParamLexer, ParamLexerError, ParamLexicalScope, ParamStatement, ParamToken, Parser, ScopedTokenizer, Tokenizer};
use crate::param::{ParamContext, ParamFile};

#[derive(Error, Debug)]
pub enum ParamParseError {
    #[error(transparent)]
    Lexical(#[from] ParamLexerError),
    #[error(transparent)]
    EncodingError(#[from] Utf8Error),
    #[error("Premature end. ")]
    PrematureEnd,
    #[error("Unknown Token.")]
    UnknownToken,
    #[error("another token was expected. ")]
    ExpectedToken

}

fn match_class(
    lexer: &mut ParamLexer,
    contextual_stack: &mut Vec<ParamClass>,
    current_token: &mut ParamToken
) -> Result<(), ParamParseError> {
    assert!(matches!(current_token, ParamToken::ClassKeyword));
    *current_token = lexer.next_token(ParamLexicalScope::Statement)?;
    let class_name = match current_token {
        ParamToken::Identifier(it) => it.clone(),
        _ => Err(ParamParseError::ExpectedToken)
    }?;
    let mut super_name: Option<Vec<u8>> = None;

    match current_token {
        ParamToken::SemiColon => {
            Ok(ParamStatement::ExternalClass(class_name))
        }
        ParamToken::Colon => {
            *current_token = lexer.next_token(ParamLexicalScope::Statement)?;

            match current_token {
                ParamToken::Identifier(it) => super_name = Some(it.clone()),
                _ => return Err(ParamParseError::ExpectedToken)
            }?
            *current_token = lexer.next_token(ParamLexicalScope::Statement)?
        }
        _ => {  }
    }?;

    if !matches!(current_token, ParamToken::LeftCurly) {
        return Err(ParamParseError::ExpectedToken)
    }

    Ok(contextual_stack.push(ParamClass {
        name: class_name,
        super_name: super_name.unwrap_or(vec![]),
        nodes: vec![],
    }))
}

impl Parser for ParamFile {
    type E = ParamParseError;

    fn try_parse(filename: String, lexer: &mut ParamLexer) -> Result<Self, Self::E> {
        let mut file = ParamFile::create(filename);
        let mut context: Vec<ParamClass>  = vec![file];

        loop {
            let mut next: ParamToken = lexer.next_token(ParamLexicalScope::Statement)?;
            let found = match next {
                ParamToken::ClassKeyword => match_class(lexer, &mut context, &mut next),
                _ => todo!()
            };

        }


        Ok(file)
    }
    
}



