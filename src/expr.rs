use crate::tokens::Token;

#[derive(Debug)]
pub(crate) enum Expr {
    Literal(Token),
    Grouping(Box<Expr>),
    Unary(Token, Box<Expr>),
    Binary(Box<Expr>, Token, Box<Expr>),
}

// TODO: Add more functions as variants are added to Expr
pub(crate) trait Visitor<T> {
    fn visit_expr(&mut self, _expr: &Expr) -> T {
        unimplemented!()
    }

    fn visit_literal(&mut self, _expr: &Expr, _lit: &Token) -> T {
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
}

impl Expr {
    pub fn accept<T>(&self, v: &mut dyn Visitor<T>) -> T {
        use Expr::*;

        match self {
            Literal(ref lit) => v.visit_literal(self, lit),
            Grouping(ref group) => v.visit_grouping(self, group),
            Unary(ref op, ref rhs) => v.visit_unary(self, op, rhs),
            Binary(ref lhs, ref op, ref rhs) => v.visit_binary(self, lhs, op, rhs),
        }
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Literal(ref lit) => write!(f, "{}", lit),
            Expr::Grouping(ref group) => write!(f, "(group {})", group),
            Expr::Unary(ref op, ref rhs) => write!(f, "({} {})", op, rhs),
            Expr::Binary(ref lhs, ref op, ref rhs) => write!(f, "({} {} {})", op, lhs, rhs),
        }
    }
}
