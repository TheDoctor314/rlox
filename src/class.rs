use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    error::{Result, RloxError},
    functions::Callable,
    object::Object,
    tokens::Token,
};

#[derive(Debug, Copy, Clone)]
pub(crate) enum ClassType {
    None,
    Class,
}
#[derive(Debug, Clone)]
pub(crate) struct LoxClass {
    name: String,
    parent: Option<Rc<LoxClass>>,
    methods: HashMap<String, Callable>,
}

impl LoxClass {
    pub(crate) fn new(
        name: String,
        parent: Option<Rc<LoxClass>>,
        methods: HashMap<String, Callable>,
    ) -> Self {
        Self {
            name,
            parent,
            methods,
        }
    }

    pub(crate) fn find_method(&self, name: &str) -> Option<&Callable> {
        if let Some(m) = self.methods.get(name) {
            return Some(m);
        } else if let Some(ref p) = self.parent {
            return p.find_method(name);
        }

        None
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

        if let Some(method) = self.class.find_method(field.lexeme.as_ref()) {
            return Ok(Object::Func(method.bind(self)));
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

use lazy_static::lazy_static;

lazy_static! {
    pub(crate) static ref THIS: Token = Token {
        token_type: crate::tokens::TokenType::This,
        lexeme: "this".to_string(),
        ..Token::default()
    };
}
