use std::collections::HashMap;

use crate::error::{Result, RloxError};
use crate::object::Object;
use crate::tokens::Token;

pub(crate) struct Env {
    values: HashMap<String, Object>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, id: &Token, val: Object) -> Result<()> {
        let name = &id.lexeme;

        self.values.insert(name.to_string(), val);
        Ok(())
    }

    pub fn assign(&mut self, id: &Token, val: Object) -> Result<Object> {
        let name = &id.lexeme;

        if !self.values.contains_key(name) {
            return Err(RloxError::Runtime(
                id.line,
                format!("Undefined variable {}", name),
                name.to_string(),
            ));
        }

        self.values.insert(name.to_string(), val.clone());
        Ok(val)
    }

    pub fn get(&mut self, id: &Token) -> Result<Object> {
        let name = &id.lexeme;

        if !self.values.contains_key(name) {
            return Err(RloxError::Runtime(
                id.line,
                format!("Undefined variable {}", name),
                name.to_string(),
            ));
        }

        Ok(self.values.get(name).cloned().unwrap())
    }
}
