use std::{collections::HashMap, rc::Rc};

use crate::{
    class::{LoxClass, SUPER, THIS},
    env::Env,
    error::{Result, RloxError},
    expr::{Expr, Visitor as ExprVisitor},
    functions::Callable,
    object::Object,
    stmt::{Stmt, Visitor as StmtVisitor},
    tokens::{Literal, Token},
};
use Object::Literal as ObjLit;

pub(crate) struct Interpreter {
    pub(crate) repl: bool,
    pub(crate) env: Rc<Env>,
    locals: Rc<HashMap<Expr, usize>>,
}

impl ExprVisitor<Result<Object>> for Interpreter {
    fn visit_expr(&mut self, _expr: &Expr) -> Result<Object> {
        unimplemented!()
    }

    fn visit_literal(&mut self, _expr: &Expr, lit: &Token) -> Result<Object> {
        Ok(ObjLit(lit.literal.as_ref().unwrap().clone()))
    }

    fn visit_logical(
        &mut self,
        _expr: &Expr,
        lhs: &Expr,
        op: &Token,
        rhs: &Expr,
    ) -> Result<Object> {
        use crate::tokens::TokenType::*;

        let lhs = lhs.accept(self)?;

        // Lox returns the operands themselves
        match op.token_type {
            Or if lhs.is_truthy() => Ok(lhs),
            And if !lhs.is_truthy() => Ok(lhs),
            _ => rhs.accept(self),
        }
    }

    fn visit_grouping(&mut self, _expr: &Expr, group: &Expr) -> Result<Object> {
        group.accept(self)
    }

    fn visit_unary(&mut self, _expr: &Expr, op: &Token, rhs: &Expr) -> Result<Object> {
        use crate::tokens::TokenType::*;
        let rhs = rhs.accept(self)?;

        match op.token_type {
            Minus => match rhs {
                ObjLit(Literal::Number(n)) => Ok(ObjLit(Literal::Number(-n))),
                _ => self.err_near("Cannot negate non-numeric value", op, format!("{:?}", rhs)),
            },
            Bang => Ok(ObjLit(Literal::Boolean(!rhs.is_truthy()))),
            _ => self.err_op("Invalid unary operator", op),
        }
    }

    fn visit_binary(&mut self, _expr: &Expr, lhs: &Expr, op: &Token, rhs: &Expr) -> Result<Object> {
        use std::cmp::Ordering;

        use crate::tokens::TokenType::*;

        let lhs = lhs.accept(self)?;
        let rhs = rhs.accept(self)?;

        let result = match op.token_type {
            Plus => match (lhs, rhs) {
                (ObjLit(Literal::Number(left_num)), ObjLit(Literal::Number(right_num))) => {
                    Literal::Number(left_num + right_num)
                }
                (ObjLit(Literal::String(ref ls)), ObjLit(ref r)) => {
                    Literal::String(format!("{}{}", ls, r))
                }
                (ObjLit(ref l), ObjLit(Literal::String(ref rs))) => {
                    Literal::String(format!("{}{}", l, rs))
                }

                (_l, _r) => {
                    return self.err_near(
                        "Cannot add mixed types",
                        op,
                        format!("{:?} + {:?}", _l, _r),
                    )
                }
            },
            Minus => match (lhs, rhs) {
                (ObjLit(Literal::Number(left_num)), ObjLit(Literal::Number(right_num))) => {
                    Literal::Number(left_num - right_num)
                }

                (_l, _r) => {
                    return self.err_near(
                        "Cannot subtract non-numeric operands",
                        op,
                        format!("{:?} - {:?}", _l, _r),
                    )
                }
            },

            Star => match (lhs, rhs) {
                (ObjLit(Literal::Number(left_num)), ObjLit(Literal::Number(right_num))) => {
                    Literal::Number(left_num * right_num)
                }

                (_l, _r) => {
                    return self.err_near(
                        "Cannot multiply non-numeric operands",
                        op,
                        format!("{:?} * {:?}", _l, _r),
                    )
                }
            },

            Slash => match (lhs, rhs) {
                (ObjLit(Literal::Number(left_num)), ObjLit(Literal::Number(right_num)))
                    if right_num == 0.0 =>
                {
                    return self.err_near(
                        "Divide by zero!! Fucker!",
                        op,
                        format!("{:?} / {:?}", left_num, right_num),
                    )
                }
                (ObjLit(Literal::Number(left_num)), ObjLit(Literal::Number(right_num))) => {
                    Literal::Number(left_num / right_num)
                }

                (_l, _r) => {
                    return self.err_near(
                        "Cannot divide non-numerics",
                        op,
                        format!("{:?} / {:?}", _l, _r),
                    )
                }
            },

            Greater | GreaterEqual | Less | LessEqual => match lhs.partial_cmp(&rhs) {
                Some(Ordering::Less) => Literal::Boolean(op.in_types(&[Less, LessEqual])),
                Some(Ordering::Equal) => Literal::Boolean(op.in_types(&[LessEqual, GreaterEqual])),
                Some(Ordering::Greater) => Literal::Boolean(op.in_types(&[Greater, GreaterEqual])),
                None => {
                    return self.err_near(
                        "Cannot compare types",
                        op,
                        format!("{:?} (compare) {:?}", lhs, rhs),
                    )
                }
            },

            EqualEqual => Literal::Boolean(lhs.eq(&rhs)),
            BangEqual => Literal::Boolean(lhs.ne(&rhs)),

            _ => return self.err_op("Invalid binary operator", op),
        };

        Ok(ObjLit(result))
    }

    fn visit_assignment(&mut self, _expr: &Expr, id: &Token, val: &Expr) -> Result<Object> {
        let v = val.accept(self)?;
        self.env.assign_at(id, v, self.locals.get(val).copied())
    }

    fn visit_identifier(&mut self, expr: &Expr, id: &Token) -> Result<Object> {
        self.lookup_var(id, expr)
    }

    fn visit_call(
        &mut self,
        _expr: &Expr,
        callee: &Expr,
        paren: &Token,
        args: &[Expr],
    ) -> Result<Object> {
        match callee.accept(self)? {
            Object::Func(ref f) => self.call_dispatch(f, paren, args),
            Object::Class(ref cls) => self.call_dispatch(&Callable::init(cls), paren, args),
            x => self.err_near(
                "Can only call functions and classes",
                paren,
                format!("{:?}", x),
            ),
        }
    }

    fn visit_get(&mut self, _expr: &Expr, callee: &Expr, prop: &Token) -> Result<Object> {
        if let Object::Instance(ref inst) = callee.accept(self)? {
            return inst.get(prop);
        }

        Err(RloxError::Runtime(
            prop.line,
            "Only instances have properties".to_string(),
            prop.lexeme.to_owned(),
        ))
    }

    fn visit_set(
        &mut self,
        _expr: &Expr,
        settee: &Expr,
        prop: &Token,
        val: &Expr,
    ) -> Result<Object> {
        if let Object::Instance(ref inst) = settee.accept(self)? {
            inst.set(prop, val.accept(self)?)
        } else {
            Err(RloxError::Runtime(
                prop.line,
                "Only instances have fields".to_string(),
                prop.lexeme.to_owned(),
            ))
        }
    }

    fn visit_this(&mut self, expr: &Expr, token: &Token) -> Result<Object> {
        self.lookup_var(token, expr)
    }
}

impl StmtVisitor<Result<()>> for Interpreter {
    fn visit_stmt(&mut self, _stmt: &Stmt) -> Result<()> {
        unimplemented!()
    }

    fn visit_expr_stmt(&mut self, stmt: &Stmt, expr: &Expr) -> Result<()> {
        if self.repl {
            self.visit_print(stmt, expr)
        } else {
            expr.accept(self).map(|_| ())
        }
    }

    fn visit_print(&mut self, _stmt: &Stmt, expr: &Expr) -> Result<()> {
        let val = expr.accept(self)?;
        println!("{}", val);

        Ok(())
    }

    fn visit_decl(&mut self, _stmt: &Stmt, id: &Token, init_expr: Option<&Expr>) -> Result<()> {
        // Returns a default of Nil if init_expr is None
        let val = init_expr.map_or_else(|| Ok(ObjLit(Literal::Nil)), |e| e.accept(self))?;

        self.env.define(id, val)
    }

    fn visit_block(&mut self, _stmt: &Stmt, body: &[Stmt]) -> Result<()> {
        let mut new_scope = self.create_scope();
        for stmt in body {
            stmt.accept(&mut new_scope)?;
        }
        Ok(())
    }

    fn visit_if(
        &mut self,
        _stmt: &Stmt,
        cond: &Expr,
        then: &Stmt,
        else_stmt: Option<&Stmt>,
    ) -> Result<()> {
        if cond.accept(self)?.is_truthy() {
            return then.accept(self);
        }

        if let Some(else_stmt) = else_stmt {
            else_stmt.accept(self)?;
        }

        Ok(())
    }

    fn visit_while(&mut self, _stmt: &Stmt, cond: &Expr, body: &Stmt) -> Result<()> {
        while cond.accept(self)?.is_truthy() {
            match body.accept(self) {
                Err(RloxError::Break(_)) => return Ok(()),
                Err(e) => return Err(e),
                _ => (),
            };
        }

        Ok(())
    }

    fn visit_break(&mut self, _stmt: &Stmt, token: &Token) -> Result<()> {
        //It should be an error to visit a break statement
        Err(RloxError::Break(token.line))
    }

    fn visit_func(
        &mut self,
        _stmt: &Stmt,
        name: &Token,
        params: &[Token],
        body: &Stmt,
    ) -> Result<()> {
        let f = Callable::new(&self.env, params, body, false);
        self.env.define(name, Object::Func(f))
    }

    fn visit_return(&mut self, _stmt: &Stmt, keyword: &Token, val: Option<&Expr>) -> Result<()> {
        let ret = match val {
            Some(expr) => expr.accept(self)?,
            None => ObjLit(Literal::Nil),
        };

        Err(RloxError::Return(keyword.line, ret))
    }

    fn visit_class(
        &mut self,
        _stmt: &Stmt,
        name: &Token,
        parent: Option<&Expr>,
        methods: &[Stmt],
    ) -> Result<()> {
        let env = Env::from(&self.env);

        let super_class = if let Some(p) = parent {
            let class = match p.accept(self)? {
                Object::Class(ref c) => Rc::clone(c),
                _ => {
                    return Err(RloxError::Runtime(
                        name.line,
                        "Superclass must be a class".to_string(),
                        name.lexeme.to_owned(),
                    ))
                }
            };

            env.define(&SUPER, Object::Class(Rc::clone(&class)))?;

            Some(class)
        } else {
            None
        };

        let mut method_map = HashMap::with_capacity(methods.len());
        for method in methods {
            match method {
                Stmt::Function(ref id, ref params, ref body) => {
                    let f = Callable::new(&env, params, body, id.lexeme.eq("init"));

                    method_map.insert(id.lexeme.clone(), f);
                }
                _ => unreachable!(),
            }
        }

        let cls = Rc::new(LoxClass::new(name.lexeme.clone(), super_class, method_map));
        self.env.define(name, Object::Class(cls))
    }
}

impl Interpreter {
    fn create_scope(&self) -> Self {
        Self {
            repl: false,
            env: Env::from(&self.env),
            locals: Rc::clone(&self.locals),
        }
    }

    fn lookup_var(&self, id: &Token, expr: &Expr) -> Result<Object> {
        self.env.get_at(id, self.locals.get(expr).copied())
    }
    fn err_near(&self, msg: &str, op: &Token, near: String) -> Result<Object> {
        Err(RloxError::Runtime(op.line, msg.to_string(), near))
    }

    // for incorrect op
    fn err_op(&self, msg: &str, op: &Token) -> Result<Object> {
        Err(RloxError::Runtime(
            op.line,
            msg.to_string(),
            op.lexeme.clone(),
        ))
    }

    fn call_dispatch(&mut self, callee: &Callable, paren: &Token, args: &[Expr]) -> Result<Object> {
        if callee.arity() != args.len() {
            return self.err_near(
                &format!(
                    "expected {} arguments but got {}",
                    callee.arity(),
                    args.len()
                ),
                paren,
                "".to_string(),
            );
        }

        let mut params = Vec::with_capacity(args.len());
        for arg in args {
            params.push(arg.accept(self)?);
        }

        callee.call(self, &params)
    }

    pub fn new(repl: bool) -> Self {
        Self {
            repl,
            env: Env::new(),
            locals: Rc::new(HashMap::new()),
        }
    }

    pub fn with_env(&self, env: Rc<Env>) -> Self {
        Self {
            repl: self.repl,
            env,
            locals: Rc::clone(&self.locals),
        }
    }

    pub(crate) fn resolve(&mut self, expr: &Expr, idx: usize) {
        Rc::get_mut(&mut self.locals)
            .expect("Should be the only mutable ref")
            .insert(expr.clone(), idx);
    }
}
