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

fn current_context(
    file: &mut ParamFile,
    context_stack: &mut Vec<ParamClass>
) -> &mut dyn ParamContext  {
    match context_stack.last_mut() {
        None => {file as &mut dyn ParamContext},
        Some(it) => {it as &mut dyn ParamContext }
    }
}

fn match_class(
    lexer: &mut ParamLexer,
    file: &mut ParamFile,
    context_stack: &mut Vec<ParamClass>,
    current_token: &mut ParamToken
) -> Result<(), ParamParseError> {
    assert!(matches!(current_token, ParamToken::ClassKeyword));
    *current_token = lexer.next_token(ParamLexicalScope::Statement)?;
    let class_name = current_token.identifier_or_err(ParamParseError::ExpectedToken)?.clone();
    let mut super_name: Vec<u8>;

    match current_token {
        ParamToken::SemiColon => {
            current_context(file, context_stack).add_node(ParamStatement::ExternalClass(class_name));
            return Ok(())
        }
        ParamToken::Colon => {
            *current_token = lexer.next_token(ParamLexicalScope::Statement)?;
            super_name = current_token.identifier_or_err(ParamParseError::ExpectedToken)?.clone();
            *current_token = lexer.next_token(ParamLexicalScope::Statement)?
        }
        _ => {  }
    }?;

    if !matches!(current_token, ParamToken::LeftCurly) {
        return Err(ParamParseError::ExpectedToken)
    }
    context_stack.push(ParamClass {
        name: class_name,
        super_name: super_name.unwrap_or(vec![]),
        nodes: vec![],
    });
    Ok(())
}

fn close_context(
    lexer: &mut ParamLexer,
    file: &mut ParamFile,
    context_stack: &mut Vec<ParamClass>,
    current_token: &mut ParamToken
) -> Result<(), ParamParseError> {
    assert!(matches!(current_token, ParamToken::RightCurly));
    *current_token = lexer.next_token(ParamLexicalScope::Statement)?;
    if context_stack.is_empty() {
        return Err(ParamParseError::UnknownToken) //Closing nothing
    }
    return match current_token {
        ParamToken::SemiColon => {
            let class = context_stack.pop().unwrap();
            Ok(current_context(file, context_stack).add_node(ParamStatement::Class(class)))
        }
        _ =>   Err(ParamParseError::ExpectedToken)
    }
}


impl Parser for ParamFile {
    type E = ParamParseError;

    fn try_parse(filename: String, lexer: &mut ParamLexer) -> Result<Self, Self::E> {
        let mut file = ParamFile::create(filename);
        let mut context_stack: Vec<ParamClass>  = vec![file];

        loop {
            let mut next: ParamToken = lexer.next_token(ParamLexicalScope::Statement)?;
            let found = match next {
                ParamToken::ClassKeyword => match_class(lexer, &mut file, &mut context_stack, &mut next),
                ParamToken::RightCurly => close_context(lexer,  &mut file, &mut context_stack, &mut next),
                _ => todo!()
            };

        }


        Ok(file)
    }
    
}

