use std::io::Error;
use thiserror::Error;
use crate::{LexicalError, LexToken};

#[derive(Error, Debug)]
pub enum PreprocessError {
    #[error(transparent)]
    IO(#[from] Error),
    #[error(transparent)]
    LexicalError(#[from] LexicalError),
    #[error(transparent)]
    Macro(#[from] MacroError),
    #[error("[{line}] Invalid preprocessor directive \"{directive_text}\"")]
    InvalidDirective {
        line:           u32,
        directive_text: String,
    },
    #[error("[{line}] Invalid preprocessor token \"{token:?}\"")]
    InvalidToken {
        line:           u32,
        token: LexToken,
    },
    #[error("Missing EndIf for IfDef/IfNDef on line {0}")]
    MissingEndIf(u32),
    #[error("Found multiple else directives for IfDef/IfNDef on line {0}")]
    MultipleElseDirectives(u32),
    #[error("[{0}] Include references an empty path.")]
    EmptyInclude(u32),
    #[error("[{0}] Found endif directive outside of if block")]
    WierdEndif(u32),
    #[error("[{0}] Found else directive outside of if block")]
    WierdElse(u32),
    #[error("Unknown Error Occurred")]
    Unknown
}

#[derive(Error, Debug)]
pub enum MacroError {
    #[error("Couldn't find macro parameter {0}")]
    UnknownMacroParameter(String),
    #[error("Couldn't find macro named {0}")]
    UnknownMacro(String),
    #[error("The parameter {0} already exists in the macro")]
    MacroParameterExists(String),
    #[error("A macro named {0} already exists in the current context.")]
    MacroExists(String),
    #[error("Could not expand macro {0} with {1} arguments, {0} expects {2}.")]
    InvalidParameterCount(String, usize, usize),
    #[error("The parameter name {0} is invalid. ")]
    InvalidMacroParameterName(String),
}

macro_rules! invalid_directive {
    ($current_line:ident, $directive_text:ident) => {
        Err(PreprocessError::InvalidDirective {
            line: $current_line,
            directive_text: $directive_text
        });
    };
}