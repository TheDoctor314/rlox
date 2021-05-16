use std::collections::HashMap;

use crate::{
    error::{Result, RloxError},
    expr::{Expr, Visitor as ExprVisitor},
    functions::FunctionType,
    interpreter::Interpreter,
    stmt::{Stmt, Visitor as StmtVisitor},
    tokens::Token,
};

pub(crate) struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    current_func: FunctionType,
    in_loop: bool,
}

impl<'a> Resolver<'a> {
    fn new(i: &'a mut Interpreter) -> Self {
        Self {
            interpreter: i,
            scopes: Vec::new(),
            current_func: FunctionType::None,
            in_loop: false,
        }
    }

    pub fn resolve(i: &'a mut Interpreter, stmt: &Stmt) -> Result<&'a mut Interpreter> {
        let mut res = Self::new(i);
        stmt.accept(&mut res)?;
        Ok(res.interpreter)
    }
}

impl<'a> ExprVisitor<Result<()>> for Resolver<'a> {
    fn visit_expr(&mut self, _expr: &Expr) -> Result<()> {
        unimplemented!()
    }

    fn visit_identifier(&mut self, expr: &Expr, id: &Token) -> Result<()> {
        // Is the variable being accessed inside its own initializer??
        let own_init = self
            .scopes
            .last()
            .and_then(|s| s.get(&id.lexeme))
            .map_or(false, |is_defined| !*is_defined);

        if own_init {
            return Err(RloxError::Parse(
                id.line,
                "Cannot read local variable in its own initializer".to_string(),
                id.lexeme.clone(),
            ));
        }

        self.resolve_local(id, expr);

        Ok(())
    }

    fn visit_literal(&mut self, _expr: &Expr, _lit: &Token) -> Result<()> {
        Ok(())
    }

    fn visit_logical(&mut self, _expr: &Expr, lhs: &Expr, _op: &Token, rhs: &Expr) -> Result<()> {
        lhs.accept(self)?;
        rhs.accept(self)
    }

    fn visit_grouping(&mut self, _expr: &Expr, group: &Expr) -> Result<()> {
        group.accept(self)
    }

    fn visit_unary(&mut self, _expr: &Expr, _op: &Token, rhs: &Expr) -> Result<()> {
        rhs.accept(self)
    }

    fn visit_binary(&mut self, _expr: &Expr, lhs: &Expr, _op: &Token, rhs: &Expr) -> Result<()> {
        lhs.accept(self)?;
        rhs.accept(self)
    }

    fn visit_assignment(&mut self, expr: &Expr, id: &Token, val: &Expr) -> Result<()> {
        val.accept(self)?;
        self.resolve_local(id, expr);
        Ok(())
    }

    fn visit_call(
        &mut self,
        _expr: &Expr,
        callee: &Expr,
        _paren: &Token,
        args: &[Expr],
    ) -> Result<()> {
        callee.accept(self)?;

        for arg in args {
            arg.accept(self)?;
        }

        Ok(())
    }

    fn visit_get(&mut self, _expr: &Expr, callee: &Expr, _prop: &Token) -> Result<()> {
        callee.accept(self)
    }

    fn visit_set(&mut self, _expr: &Expr, settee: &Expr, _prop: &Token, val: &Expr) -> Result<()> {
        val.accept(self)?;
        settee.accept(self)
    }
}

impl<'a> StmtVisitor<Result<()>> for Resolver<'a> {
    fn visit_stmt(&mut self, _stmt: &Stmt) -> Result<()> {
        unimplemented!()
    }

    fn visit_expr_stmt(&mut self, _stmt: &Stmt, expr: &Expr) -> Result<()> {
        expr.accept(self)
    }

    fn visit_print(&mut self, _stmt: &Stmt, expr: &Expr) -> Result<()> {
        expr.accept(self)
    }

    fn visit_decl(&mut self, _stmt: &Stmt, id: &Token, init_expr: Option<&Expr>) -> Result<()> {
        self.declare(id)?;

        if let Some(expr) = init_expr {
            expr.accept(self)?;
        }

        self.define(id)
    }

    fn visit_block(&mut self, _stmt: &Stmt, body: &[Stmt]) -> Result<()> {
        self.begin_scope();

        for stmt in body {
            stmt.accept(self)?;
        }

        self.end_scope();
        Ok(())
    }

    fn visit_if(
        &mut self,
        _stmt: &Stmt,
        cond: &Expr,
        then: &Stmt,
        else_stmt: Option<&Stmt>,
    ) -> Result<()> {
        cond.accept(self)?;
        then.accept(self)?;
        if let Some(stmt) = else_stmt {
            stmt.accept(self)?;
        }

        Ok(())
    }

    fn visit_while(&mut self, _stmt: &Stmt, cond: &Expr, body: &Stmt) -> Result<()> {
        let prev = self.in_loop;
        self.in_loop = true;

        cond.accept(self)?;
        body.accept(self)?;

        self.in_loop = prev;
        Ok(())
    }

    fn visit_break(&mut self, _stmt: &Stmt, token: &Token) -> Result<()> {
        if !self.in_loop {
            return Err(RloxError::Break(token.line));
        }
        Ok(())
    }

    fn visit_func(
        &mut self,
        _stmt: &Stmt,
        name: &Token,
        params: &[Token],
        body: &Stmt,
    ) -> Result<()> {
        self.declare(name)?;
        self.define(name)?;

        self.resolve_function(params, body, FunctionType::Function)
    }

    fn visit_return(&mut self, _stmt: &Stmt, keyword: &Token, val: Option<&Expr>) -> Result<()> {
        if let FunctionType::None = self.current_func {
            return Err(RloxError::Parse(
                keyword.line,
                "cannot return from top-level code".to_string(),
                keyword.lexeme.to_owned(),
            ));
        }

        if let Some(val) = val {
            val.accept(self)?;
        }

        Ok(())
    }

    fn visit_class(&mut self, _stmt: &Stmt, name: &Token, methods: &[Stmt]) -> Result<()> {
        self.declare(name)?;
        self.define(name)?;

        for method in methods {
            match method {
                Stmt::Function(ref _id, ref params, ref body) => {
                    let func_type = FunctionType::Method;
                    self.resolve_function(params, body.as_ref(), func_type)?;
                }
                _ => unreachable!(),
            }
        }

        Ok(())
    }
}

impl<'a> Resolver<'a> {
    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, id: &Token) -> Result<()> {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.insert(id.lexeme.to_owned(), false).is_some() {
                return Err(RloxError::Parse(
                    id.line,
                    "variable already defined with this name in this scope".to_string(),
                    id.lexeme.to_owned(),
                ));
            }
        }

        Ok(())
    }

    fn define(&mut self, id: &Token) -> Result<()> {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(id.lexeme.to_owned(), true);
        }

        Ok(())
    }

    fn resolve_local(&mut self, id: &Token, expr: &Expr) {
        let len = self.scopes.len();
        for i in (0..len).rev() {
            if self.scopes[i].contains_key(&id.lexeme) {
                self.interpreter.resolve(expr, len - (i + 1));
                return;
            }
        }
    }

    fn resolve_function(
        &mut self,
        params: &[Token],
        body: &Stmt,
        func_type: FunctionType,
    ) -> Result<()> {
        let prev_type = self.current_func;
        self.current_func = func_type;
        self.begin_scope();

        for param in params {
            self.declare(param)?;
        }

        body.accept(self)?;
        self.end_scope();
        self.current_func = prev_type;

        Ok(())
    }
}
