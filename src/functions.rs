use std::rc::Rc;

use crate::{
    class::{LoxClass, LoxInstance, THIS},
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
    Initializer,
    Method,
}

#[derive(Debug, Clone)]
pub(crate) enum Callable {
    Runtime(LoxFunction),
    Init(ClassInit),
}

impl Callable {
    pub fn new(env: &Rc<Env>, params: &[Token], body: &Stmt, init: bool) -> Self {
        Callable::Runtime(LoxFunction::new(env, params, body, init))
    }

    pub fn init(class: &Rc<LoxClass>) -> Self {
        Callable::Init(ClassInit(Rc::clone(class)))
    }

    pub fn arity(&self) -> usize {
        match self {
            Callable::Runtime(ref f) => f.arity(),
            Callable::Init(ref cls) => cls.arity(),
        }
    }

    pub fn call(&self, interpreter: &Interpreter, args: &[Object]) -> Result<Object> {
        match self {
            Callable::Runtime(ref f) => f.call(interpreter, args),
            Callable::Init(ref cls) => cls.call(interpreter, args),
        }
    }

    pub fn bind(&self, inst: &LoxInstance) -> Self {
        match self {
            Callable::Runtime(ref f) => Callable::Runtime(f.bind(inst)),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LoxFunction {
    closure: Rc<Env>,
    params: Vec<Token>,
    body: Box<Stmt>,
    init: bool,
}

impl LoxFunction {
    pub fn new(scope: &Rc<Env>, params: &[Token], body: &Stmt, init: bool) -> Self {
        Self {
            closure: Rc::clone(scope),
            params: params.to_vec(),
            body: Box::new(body.clone()),
            init,
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
            Ok(()) | Err(RloxError::Return(_, _)) if self.init => {
                self.closure.get_at(&THIS, Some(0))
            }
            Ok(()) => Ok(Object::Literal(Nil)),
            Err(RloxError::Return(_, ret)) => Ok(ret),
            Err(e) => Err(e),
        }
    }

    pub fn bind(&self, inst: &LoxInstance) -> Self {
        let env = Env::from(&self.closure);
        env.define(&THIS, Object::Instance(inst.clone()))
            .expect("Failed to define 'this'");

        Self::new(&env, &self.params, &self.body, self.init)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ClassInit(Rc<LoxClass>);

impl ClassInit {
    pub fn arity(&self) -> usize {
        self.0.find_method("init").map_or(0, |init| init.arity())
    }

    pub fn call(&self, interpreter: &Interpreter, args: &[Object]) -> Result<Object> {
        let inst = LoxInstance::new(&self.0);

        if let Some(init) = self.0.find_method("init") {
            init.bind(&inst).call(interpreter, args)?;
        }

        Ok(Object::Instance(inst))
    }
}
