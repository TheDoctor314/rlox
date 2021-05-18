use crate::tokens::Token;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Expr {
    Identifier(Token),
    Literal(Token),
    Logical(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Unary(Token, Box<Expr>),
    Binary(Box<Expr>, Token, Box<Expr>),
    Assignment(Token, Box<Expr>),
    Call(Box<Expr>, Token, Vec<Expr>),
    Get(Box<Expr>, Token),
    Set(Box<Expr>, Token, Box<Expr>),
    This(Token),
    Super(Token, Token),
}

// TODO: Add more functions as variants are added to Expr
pub(crate) trait Visitor<T> {
    fn visit_expr(&mut self, _expr: &Expr) -> T {
        unimplemented!()
    }

    fn visit_identifier(&mut self, _expr: &Expr, _id: &Token) -> T {
        self.visit_expr(_expr)
    }

    fn visit_literal(&mut self, _expr: &Expr, _lit: &Token) -> T {
        self.visit_expr(_expr)
    }

    fn visit_logical(&mut self, _expr: &Expr, _lhs: &Expr, _op: &Token, _rhs: &Expr) -> T {
        self.visit_expr(_expr)
    }

    fn visit_grouping(&mut self, _expr: &Expr, _group: &Expr) -> T {
        self.visit_expr(_expr)
    }

    fn visit_unary(&mut self, _expr: &Expr, _op: &Token, _rhs: &Expr) -> T {
        self.visit_expr(_expr)
    }

    fn visit_binary(&mut self, _expr: &Expr, _lhs: &Expr, _op: &Token, _rhs: &Expr) -> T {
        self.visit_expr(_expr)
    }

    fn visit_assignment(&mut self, _expr: &Expr, _id: &Token, _val: &Expr) -> T {
        self.visit_expr(_expr)
    }

    fn visit_call(&mut self, _expr: &Expr, _callee: &Expr, _paren: &Token, _args: &[Expr]) -> T {
        self.visit_expr(_expr)
    }

    fn visit_get(&mut self, _expr: &Expr, _callee: &Expr, _prop: &Token) -> T {
        self.visit_expr(_expr)
    }

    fn visit_set(&mut self, _expr: &Expr, _settee: &Expr, _prop: &Token, _val: &Expr) -> T {
        self.visit_expr(_expr)
    }

    fn visit_this(&mut self, _expr: &Expr, _token: &Token) -> T {
        self.visit_expr(_expr)
    }

    fn visit_super(&mut self, _expr: &Expr, _keyword: &Token, _method: &Token) -> T {
        self.visit_expr(_expr)
    }
}

impl Expr {
    pub fn accept<T>(&self, v: &mut dyn Visitor<T>) -> T {
        use Expr::*;

        match self {
            Identifier(ref id) => v.visit_identifier(self, id),
            Literal(ref lit) => v.visit_literal(self, lit),
            Logical(ref lhs, ref op, ref rhs) => v.visit_logical(self, lhs, op, rhs),
            Grouping(ref group) => v.visit_grouping(self, group),
            Unary(ref op, ref rhs) => v.visit_unary(self, op, rhs),
            Binary(ref lhs, ref op, ref rhs) => v.visit_binary(self, lhs, op, rhs),
            Assignment(ref id, ref val) => v.visit_assignment(self, id, val),
            Call(ref callee, ref paren, ref args) => {
                v.visit_call(self, callee.as_ref(), paren, args)
            }
            Get(ref callee, ref prop) => v.visit_get(self, callee.as_ref(), prop),
            Set(ref settee, ref prop, ref val) => {
                v.visit_set(self, settee.as_ref(), prop, val.as_ref())
            }
            This(ref token) => v.visit_this(self, token),
            Super(ref token, ref method) => v.visit_super(self, token, method),
        }
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Identifier(ref id) => write!(f, "{}", id),
            Expr::Literal(ref lit) => write!(f, "{}", lit),
            Expr::Logical(ref lhs, ref op, ref rhs) => write!(f, "({} {} {})", op, lhs, rhs),
            Expr::Grouping(ref group) => write!(f, "(group {})", group),
            Expr::Unary(ref op, ref rhs) => write!(f, "({} {})", op, rhs),
            Expr::Binary(ref lhs, ref op, ref rhs) => write!(f, "({} {} {})", op, lhs, rhs),
            Expr::Assignment(ref id, ref val) => write!(f, "(= {} {})", id, val),
            Expr::Call(ref callee, _, ref args) => write!(f, "{}({:?})", callee, args),
            Expr::Get(ref callee, ref prop) => write!(f, "{}.{}", callee, prop),
            Expr::Set(ref settee, ref prop, ref val) => {
                write!(f, "{}.{} = {}", settee.as_ref(), prop, val.as_ref())
            }
            Expr::This(_) => write!(f, "this"),
            Expr::Super(_, ref method) => write!(f, "super.{}", method.lexeme),
        }
    }
}
