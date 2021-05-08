use std::rc::Rc;

use crate::object::Object;
use crate::stmt::{Stmt, Visitor as StmtVisitor};
use crate::tokens::Token;
use crate::{
    env::Env,
    error::{Result, RloxError},
};
use crate::{
    expr::{Expr, Visitor as ExprVisitor},
    tokens::Literal,
};
use Object::Literal as ObjLit;

pub(crate) struct Interpreter {
    repl: bool,
    env: Rc<Env>,
}

impl ExprVisitor<Result<Object>> for Interpreter {
    fn visit_expr(&mut self, _expr: &Expr) -> Result<Object> {
        unimplemented!()
    }

    fn visit_literal(&mut self, _expr: &Expr, lit: &Token) -> Result<Object> {
        Ok(ObjLit(lit.literal.as_ref().unwrap().clone()))
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
                (ObjLit(Literal::String(ref ls)), ObjLit(Literal::String(ref rs))) => {
                    Literal::String(format!("{}{}", ls, rs))
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
        self.env.assign(id, v)
    }

    fn visit_identifier(&mut self, expr: &Expr, id: &Token) -> Result<Object> {
        self.lookup_var(id, expr)
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
        println!("{:?}", val);

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

    fn visit_if(&mut self, _stmt: &Stmt, cond: &Expr, then: &Stmt, else_stmt: Option<&Stmt>) -> Result<()> {
        if cond.accept(self)?.is_truthy() {
            return then.accept(self);
        }

        if let Some(else_stmt) = else_stmt {
            else_stmt.accept(self)?;
        }

        Ok(())
    }
}

impl Interpreter {
    fn create_scope(&self) -> Self {
        Self {
            repl: false,
            env: Env::from(&self.env),
        }
    }

    fn lookup_var(&mut self, id: &Token, _expr: &Expr) -> Result<Object> {
        self.env.get(id)
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

    pub(crate) fn interpret(&mut self, stmt: &Stmt) -> Result<()> {
        stmt.accept(self)
    }

    pub fn new(repl: bool) -> Self {
        Self {
            repl,
            env: Env::new(),
        }
    }
}
