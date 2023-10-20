pub mod lexer;

use std::collections::HashMap;
use std::hash::Hash;
pub use lexer::*;
pub mod parser; pub use parser::*;

type StatementId = u32;
type ConstantId = u32;
type ContextId = u32;
type ExpressionId = u32;

type StatementGroup = Box<Vec<StatementId>>;
type ExpressionGroup = Box<Vec<ExpressionId>>;

const FILE_ROOT: ContextId = 0;

pub struct ParamConstant {
    name:             String,
    value:            f32
}

pub struct ParamFile {
    statements:       HashMap<StatementId, ParamStatement>,
    constants:        HashMap<ConstantId, ParamConstant>,
    contexts:         HashMap<ContextId, StatementGroup>,
    expressions:      HashMap<ExpressionId, ParamExpression>,
    name:             String,
}

pub enum ParamClassStatement {
    External(String),
    Normal {
        name:         String,
        super_class:  String,
        context_id:   ContextId
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
    Expression(ConstantId),
    Float(f32),
    Integer(i32),
    Long(i64)
}


impl ParamFile {

    pub fn create(filename: String) -> Self {
        todo!()
    }
}
