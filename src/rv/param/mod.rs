pub mod lexer;

use std::collections::HashMap;
use std::hash::Hash;
pub use lexer::*;
pub mod parser; pub use parser::*;

type NodeSet = Vec<ParamStatement>;
type ValueSet = Vec<ParamExpression>;

pub struct ParamFile {
    name:               Vec<u8>,
    internals:          HashMap<Vec<u8>, i32>,
    nodes:              NodeSet
}

#[derive(PartialEq)]
pub struct ParamClass {
    name:               Vec<u8>,
    super_name:         Vec<u8>,
    nodes:              NodeSet
}

#[derive(PartialEq)]
pub enum ParamStatement {
    ExternalClass(Vec<u8>),
    Class(ParamClass),
    Delete(Vec<u8>),
    Variable(Vec<u8>, ParamExpression)
}

#[derive(PartialEq)]
pub enum ParamExpression {
    Atomic(ParamLiteral),
    Array(ValueSet)
}

#[derive(PartialEq)]
pub enum ParamLiteral {
    String(bool, Vec<u8>),
    Float(f32),
    Integer(i32),
    Long(i64)
}

impl ParamContext for ParamClass {
    fn name(&self) -> &Vec<u8> { &self.name }

    fn nodes(&self) -> &NodeSet { &self.nodes }

    fn mut_nodes(&mut self) -> &mut NodeSet { &mut self.nodes }
}

trait ParamContext {
    fn name(&self) -> &Vec<u8>;

    fn nodes(&self) -> &NodeSet;

    fn mut_nodes(&mut self) -> &mut NodeSet;

    fn add_node(&mut self, node: ParamStatement) { self.mut_nodes().push(node); }

    fn remove_node(&mut self, index: usize) { self.mut_nodes().remove(index); }

    fn edit_node(&mut self, index: usize) -> Option<&mut ParamStatement> { self.mut_nodes().get_mut(index) }

    fn get_node(&self, index: usize) -> Option<&ParamStatement> {
        match self.nodes().get(index) {
            Some(node) => Some(node),
            None => None,
        }
    }
}



impl ParamFile {


}
