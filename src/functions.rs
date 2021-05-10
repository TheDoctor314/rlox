use std::rc::Rc;

use crate::{
    error::Result,
    interpreter::Interpreter,
    object::Object,
    stmt::Stmt,
    tokens::Token,
};

#[derive(Debug, Clone)]
pub(crate) enum Callable {
    Runtime(LoxFunction),
}

impl Callable {
    pub fn new(params: &[Token], body: &Stmt) -> Self {
        Callable::Runtime(LoxFunction::new(params, body))
    }

    pub fn arity(&self) -> usize {
        match self {
            Callable::Runtime(ref f) => f.arity(),
        }
    }

    pub fn call(&self, interpreter: &Interpreter, args: &[Object]) -> Result<Object> {
        match self {
            Callable::Runtime(ref f) => f.call(interpreter, args),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LoxFunction {
    params: Vec<Token>,
    body: Box<Stmt>,
}

impl LoxFunction {
    pub fn new(params: &[Token], body: &Stmt) -> Self {
        Self {
            params: params.to_vec(),
            body: Box::new(body.clone()),
        }
    }

    pub fn arity(&self) -> usize {
        self.params.len()
    }

    pub fn call(&self, interpreter: &Interpreter, args: &[Object]) -> Result<Object> {
        use crate::tokens::Literal::Nil;

        let env = Rc::clone(&interpreter.env);

        for(param, arg) in self.params.iter().zip(args.into_iter()) {
            env.define(param, arg.clone())?;
        }

        match self.body.accept(&mut interpreter.with_env(env)) {
            Ok(()) => Ok(Object::Literal(Nil)),
            Err(e) => Err(e),
        }
    }
}
