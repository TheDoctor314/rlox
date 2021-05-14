use std::rc::Rc;

use crate::{
    env::Env,
    error::{Result, RloxError},
    interpreter::Interpreter,
    object::Object,
    stmt::Stmt,
    tokens::Token,
};

#[derive(Debug, Copy, Clone)]
pub(crate) enum FunctionType {
    None,
    Function,
}

#[derive(Debug, Clone)]
pub(crate) enum Callable {
    Runtime(LoxFunction),
}

impl Callable {
    pub fn new(env: &Rc<Env>, params: &[Token], body: &Stmt) -> Self {
        Callable::Runtime(LoxFunction::new(env, params, body))
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
    closure: Rc<Env>,
    params: Vec<Token>,
    body: Box<Stmt>,
}

impl LoxFunction {
    pub fn new(scope: &Rc<Env>, params: &[Token], body: &Stmt) -> Self {
        Self {
            closure: Rc::clone(scope),
            params: params.to_vec(),
            body: Box::new(body.clone()),
        }
    }

    pub fn arity(&self) -> usize {
        self.params.len()
    }

    pub fn call(&self, interpreter: &Interpreter, args: &[Object]) -> Result<Object> {
        use crate::tokens::Literal::Nil;

        let env = Env::from(&self.closure);

        for (param, arg) in self.params.iter().zip(args.iter()) {
            env.define(param, arg.clone())?;
        }

        match self.body.accept(&mut interpreter.with_env(env)) {
            Ok(()) => Ok(Object::Literal(Nil)),
            Err(RloxError::Return(_, ret)) => Ok(ret),
            Err(e) => Err(e),
        }
    }
}
