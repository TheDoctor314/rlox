use std::rc::Rc;

use crate::{class::LoxClass, functions::Callable};
use crate::{class::LoxInstance, tokens};

#[derive(Debug, Clone)]
pub(crate) enum Object {
    Literal(tokens::Literal),
    Func(Callable),
    Class(Rc<LoxClass>),
    Instance(LoxInstance),
}

impl std::cmp::PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        use Object::Literal as ObjLit;
        match (self, other) {
            (&ObjLit(ref lhs), &ObjLit(ref rhs)) => lhs.eq(rhs),
            _ => false,
        }
    }
}

impl std::cmp::PartialOrd for Object {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use Object::Literal as ObjLit;
        match (self, other) {
            (&ObjLit(ref lhs), &ObjLit(ref rhs)) => lhs.partial_cmp(rhs),
            _ => None,
        }
    }
}

impl Object {
    // Lox follow Ruby's rule: false and nil are falsey
    // otherwise depends on literal
    pub fn is_truthy(&self) -> bool {
        use tokens::Literal::*;
        match self {
            Object::Literal(ref lit) => match *lit {
                Nil => false,
                Boolean(b) => b,
                Number(n) => n != 0.0,
                String(ref s) => !s.is_empty(),
            },
            _ => true,
        }
    }
}

impl std::fmt::Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Literal(ref lit) => write!(f, "{}", lit),
            Object::Func(_) => write!(f, "<function>"),
            Object::Class(ref cls) => write!(f, "{}", cls),
            Object::Instance(ref i) => write!(f, "{}", i),
        }
    }
}
