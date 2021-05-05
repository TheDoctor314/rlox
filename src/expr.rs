use crate::tokens::Token;

#[derive(Debug)]
pub(crate) enum Expr {
    Literal(Token),
    Grouping(Box<Expr>),
    Unary(Token, Box<Expr>),
    Binary(Box<Expr>, Token, Box<Expr>),
}

pub(crate) trait Visitor<T> {
    fn visit() {
        todo!()
    }
}
