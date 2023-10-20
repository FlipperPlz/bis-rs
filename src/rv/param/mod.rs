pub mod lexer;

use std::collections::HashMap;
use std::hash::Hash;
pub use lexer::*;
pub mod parser; pub use parser::*;

type StatementId = u32;
type ExpressionId = u32;
type StatementGroup = Box<Vec<StatementId>>;
type ExpressionGroup = Box<Vec<ParamExpression>>;

pub struct ParamConstant {
    name:             String,
    value:            f32
}

pub struct ParamFile {
    statements:       HashMap<StatementId, ParamStatement>,
    constants:        HashMap<ExpressionId, ParamConstant>,
    name:             String,
    root:             StatementGroup
}

pub enum ParamClassStatement {
    External(String),
    Normal {
        name:         String,
        super_class:  String,
        statements:   StatementGroup
    }
}

pub enum ParamStatement {
    Class(ParamClassStatement),
    Delete(String),
    Variable(String, ParamExpression)
}

pub enum ParamExpression {
    Literal(ParamLiteral),
    Array(ExpressionGroup)
}

pub enum ParamLiteral {
    String(bool, String),
    Expression(ExpressionId),
    Float(f32),
    Integer(i32),
    Long(i64)
}

