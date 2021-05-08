use std::rc::Rc;
use std::{cell::RefCell, collections::HashMap};

use crate::error::{Result, RloxError};
use crate::object::Object;
use crate::tokens::Token;

pub(crate) struct Env {
    parent: Option<Rc<Env>>,
    values: RefCell<HashMap<String, Object>>,
}

impl Env {
    pub fn new() -> Rc<Env> {
        Rc::new(Self {
            parent: None,
            values: RefCell::new(HashMap::new()),
        })
    }

    pub fn from(parent: &Rc<Env>) -> Rc<Env> {
        Rc::new(Self {
            parent: Some(Rc::clone(parent)),
            values: RefCell::new(HashMap::new()),
        })
    }

    pub fn define(&self, id: &Token, val: Object) -> Result<()> {
        let name = &id.lexeme;

        self.values.borrow_mut().insert(name.to_string(), val);
        Ok(())
    }

    pub fn assign(&self, id: &Token, val: Object) -> Result<Object> {
        let name = &id.lexeme;
        let mut values = self.values.borrow_mut();

        if !values.contains_key(name) {
            if let Some(ref parent) = self.parent {
                return parent.get(id);
            }

            return Err(RloxError::Runtime(
                id.line,
                format!("Undefined variable {}", name),
                name.to_string(),
            ));
        }

        values.insert(name.to_string(), val.clone());
        Ok(val)
    }

    // recursive; if not in current scope then looks in parent scope
    pub fn get(&self, id: &Token) -> Result<Object> {
        let name = &id.lexeme;
        let values = self.values.borrow_mut();

        if !values.contains_key(name) {
            if let Some(ref parent) = self.parent {
                return parent.get(id);
            }

            return Err(RloxError::Runtime(
                id.line,
                format!("Undefined variable {}", name),
                name.to_string(),
            ));
        }

        Ok(values.get(name).cloned().unwrap())
    }
}
