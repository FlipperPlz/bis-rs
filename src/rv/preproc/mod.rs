mod io;
mod error; use error::*;

use std::collections::HashMap;

type Symbol           = String;
type MacroArgument    = Symbol;
type MacroArguments   = Vec<Symbol>;
type MacroTable       = HashMap<Symbol, CustomMacro>;

struct PreProcessor {
    macros:        MacroTable
}

struct CustomMacro {
    value:         Symbol,
    parameters:    MacroArguments,
    blocked:       i32,
}

impl CustomMacro {
    pub fn new_simple(value: &str) -> Self { Self::new(value, Vec::new()) }

    pub fn new(value: &str, parameters: Vec<Symbol>) -> Self {
        Self {
            value: value.to_string(),
            parameters,
            blocked: 0,
        }
    }

    pub fn has_params(&self) -> bool { !self.parameters.is_empty() }

    pub fn blocked(&self) -> bool { return self.blocked != 0 }

    pub fn unblock(&mut self) { if self.blocked > 0 { self.blocked -= 1; } }

    pub fn block(&mut self) { self.blocked += 1; }
}

impl PreProcessor {
    pub fn add_simple_define(&mut self, name: &str, value: &str) -> Option<CustomMacro> {
        self.macros.insert(name.to_string(),CustomMacro::new_simple(value))
    }

}