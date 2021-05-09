use crate::tokens::Token;

#[derive(Debug)]
pub(crate) enum Expr {
    Identifier(Token),
    Literal(Token),
    Logical(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Unary(Token, Box<Expr>),
    Binary(Box<Expr>, Token, Box<Expr>),
    Assignment(Token, Box<Expr>),
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

    fn visit_logical(&mut self, _expr: &Expr, _lhs: &Expr, _op: &Token, _rhs: &Expr) -> T{
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
        }
    }
}
