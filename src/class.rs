use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    error::{Result, RloxError},
    object::Object,
    tokens::Token,
};

#[derive(Debug, Clone)]
pub(crate) struct LoxClass {
    name: String,
}

impl LoxClass {
    pub(crate) fn new(name: String) -> Self {
        Self { name }
    }
}
impl std::fmt::Display for LoxClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LoxInstance {
    class: Rc<LoxClass>,
    fields: Rc<RefCell<HashMap<String, Object>>>,
}

impl LoxInstance {
    pub(crate) fn new(class: &Rc<LoxClass>) -> Self {
        Self {
            class: Rc::clone(class),
            fields: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub(crate) fn get(&self, field: &Token) -> Result<Object> {
        if let Some(obj) = self.fields.borrow().get(&field.lexeme) {
            return Ok(obj.clone());
        }

        Err(RloxError::Runtime(
            field.line,
            format!("Undefined property {}", field.lexeme),
            field.lexeme.to_owned(),
        ))
    }

    // Lox allows freely creating new fields
    pub(crate) fn set(&self, field: &Token, val: Object) -> Result<Object> {
        self.fields
            .borrow_mut()
            .insert(field.lexeme.clone(), val.clone());
        Ok(val)
    }
}

impl std::fmt::Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class)
    }
}
