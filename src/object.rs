use crate::tokens;

#[derive(Debug, PartialEq, PartialOrd)]
pub(crate) enum Object {
    Literal(tokens::Literal),
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
        }
    }
}
