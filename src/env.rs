use std::rc::Rc;
use std::{cell::RefCell, collections::HashMap};

use crate::error::{Result, RloxError};
use crate::object::Object;
use crate::tokens::Token;

#[derive(Debug)]
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

    pub fn assign_at(&self, id: &Token, val: Object, dist: Option<usize>) -> Result<Object> {
        if dist.map_or(0, |d| d) == 0 {
            return self.assign(id, val);
        }

        let dist = dist.unwrap();

        if let Some(ancestor) = self.ancestor(dist) {
            return ancestor.assign(id, val);
        }

        Err(RloxError::Runtime(
            id.line,
            format!("Ancestor is undefined at depth {}", dist),
            id.lexeme.to_owned(),
        ))
    }

    // recursive; if not in current scope then looks in parent scope
    pub fn assign(&self, id: &Token, val: Object) -> Result<Object> {
        let name = &id.lexeme;
        let mut values = self.values.borrow_mut();

        if !values.contains_key(name) {
            if let Some(ref parent) = self.parent {
                return parent.assign(id, val);
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

    // looks in a specific scope for the variable or throws an error otherwise
    pub fn get_at(&self, id: &Token, dist: Option<usize>) -> Result<Object> {
        if dist.is_none() {
            // return from global scope
            return self.get_global(id);
        }

        let dist = dist.unwrap();
        if dist == 0 {
            return self.get(id);
        }

        if let Some(ancestor) = self.ancestor(dist) {
            return ancestor.get(id);
        }

        Err(RloxError::Runtime(
            id.line,
            format!("Ancestor is undefined at depth {}", dist),
            id.lexeme.to_owned(),
        ))
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

impl Env {
    fn ancestor(&self, dist: usize) -> Option<Rc<Env>> {
        let mut env = self.parent.clone();

        for _ in 1..dist {
            env = env?.parent.clone();
        }

        env
    }

    fn get_global(&self, id: &Token) -> Result<Object> {
        match self.parent {
            None => self.get(id),
            Some(ref p) => p.get_global(id),
        }
    }
}
