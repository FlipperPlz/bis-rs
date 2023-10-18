use std::cmp::Reverse;
use crate::{MacroError};

type MacroValue = String;
type MacroParam = String;
pub type MacroName = String;

pub type MacroResult<O> = Result<O, MacroError>;
pub type MacroVoidResult = MacroResult<()>;
pub struct Macro {
    params:     Vec<MacroParam>,
    contents:   MacroValue,
    blocked:    u32
}

impl Macro {
    
    pub fn create_simple(params: Vec<String>, contents: String) -> Self {
        Self {
            params,
            contents,
            blocked: 0,
        }
    }
    
    #[inline]
    pub fn takes_params(&self) -> bool { self.params.is_empty() }

    #[inline]
    pub fn remove_params(&mut self)  { self.params.clear() }

    #[inline]
    pub fn block(&mut self) { self.blocked += 1 }

    #[inline]
    pub fn unblock(&mut self) { if self.blocked > 0 {self.blocked -= 1 } }

    #[inline]
    pub fn blocked(&self) -> bool { self.blocked != 0 }

    pub fn bind_param(&mut self, param_name: String) -> MacroVoidResult {
        return if !Self::validate_param_name(&param_name) {
            Err(MacroError::UnknownMacroParameter(param_name.clone()))
        } else if self.params.contains(&param_name) {
            Err(MacroError::UnknownMacroParameter(param_name.clone()))
        } else {
            Ok(self.params.push(param_name))
        }
    }

    pub fn rename_param(&mut self, old_param: &mut MacroParam, new_name: String) -> MacroVoidResult {
        return if !self.params.contains(&old_param) {
            Err(MacroError::UnknownMacroParameter(old_param.clone()))
        } else if !Self::validate_param_name(&new_name) {
            Err(MacroError::UnknownMacroParameter(old_param.clone()))
        } else {
            self.force_rename_param(old_param, new_name)
        }
    }

    pub fn force_rename_param(&mut self, old_param: &mut MacroParam, new_name: String) -> MacroVoidResult {
        self.contents = self.contents.replace(&old_param.to_string(), &*new_name);
        Ok(*old_param = new_name)
    }

    pub fn refactor_param(&mut self, old_param: &mut MacroParam, new_name: String) -> MacroVoidResult {
        if !Self::validate_param_name(&new_name) {
            return Err(MacroError::UnknownMacroParameter(new_name.clone()))
        } else if self.params.contains(&new_name) {
            return Err(MacroError::UnknownMacroParameter(new_name.clone()))
        }
        self.contents = self.contents.replace(&old_param.to_string(), &*new_name);
        self.force_rename_param(old_param, new_name)
    }

    pub fn get_value(&self) -> &MacroValue {
        &self.contents
    }

    pub fn parameter_count(&self) -> usize { self.params.len() }

    pub fn evaluate(&self, debug_macro_name: String, parameters: Vec<String>) -> MacroResult<MacroValue> {
        if parameters.len() != self.params.len() {
            return Err(MacroError::InvalidParameterCount(debug_macro_name, parameters.len(),self.params.len() ))
        }
        let params: Vec<(String, usize)> = {
            let mut p: Vec<(String, usize)> = self.params.iter().enumerate().map(|(i, s)| (s.clone(), i)).collect();
            p.sort_by_cached_key(|a| Reverse(a.0.len()));
            p
        };
        let mut result = self.get_value().clone();
        for (param, index) in params {
            result = result.replace(&param, parameters[index].as_str())
        }
        Ok(result)
    }


    fn validate_param_name(name: &String) -> bool { /*TODO*/true }

}