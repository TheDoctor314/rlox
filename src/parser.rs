use std::iter::Peekable;

use crate::{
    error::{Result, RloxError},
    expr::Expr,
    scanner::Scanner,
    tokens::{Token, TokenType},
};
use TokenType::*;

pub(crate) struct Parser<'a> {
    src: Peekable<Scanner<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(src: Scanner<'a>) -> Self {
        Self {
            src: src.peekable(),
        }
    }

    pub fn parse(&mut self) -> Result<Expr> {
        self.expression()
    }
}

// Expression related methods
impl<'a> Parser<'a> {
    fn expression(&mut self) -> Result<Expr> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr> {
        let mut expr = self.comparison()?;

        while let Some(op) = self.check_advance(&[BangEqual, EqualEqual]) {
            expr = Expr::Binary(Box::new(expr), op?, Box::new(self.comparison()?));
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr> {
        let mut expr = self.term()?;

        while let Some(op) = self.check_advance(&[Greater, GreaterEqual, Less, LessEqual]) {
            expr = Expr::Binary(Box::new(expr), op?, Box::new(self.term()?));
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr> {
        let mut expr = self.factor()?;

        while let Some(op) = self.check_advance(&[Plus, Minus]) {
            expr = Expr::Binary(Box::new(expr), op?, Box::new(self.factor()?));
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr> {
        let mut expr = self.unary()?;

        while let Some(op) = self.check_advance(&[Slash, Star]) {
            expr = Expr::Binary(Box::new(expr), op?, Box::new(self.unary()?));
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr> {
        if let Some(op) = self.check_advance(&[Bang, Minus]) {
            return Ok(Expr::Unary(op?, Box::new(self.unary()?)));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr> {
        if let Some(Ok(token)) = self.check_advance(&[Nil, False, True, Number, StringLiteral]) {
            return match token.token_type {
                Nil | False | True | Number | StringLiteral => Ok(Expr::Literal(token)),
                _ => Err(Parser::unexpected(&token)),
            };
        }

        if let Some(Ok(_)) = self.check_advance(&[LParen]) {
            let expr = self.expression()?;
            let _rbrace = self.must_advance(&[RParen]);

            return Ok(Expr::Grouping(Box::new(expr)));
        }

        Err(self.peek_err())
    }
}

// helper token related methods
impl<'a> Parser<'a> {
    fn check(&mut self, types: &[TokenType]) -> bool {
        match self.src.peek() {
            Some(&Ok(ref t)) => t.in_types(types),
            _ => false,
        }
    }

    fn check_advance(&mut self, types: &[TokenType]) -> Option<Result<Token>> {
        if self.check(types) {
            return self.src.next();
        }

        None
    }

    // This function returns an error if it's not possible to advance
    fn must_advance(&mut self, types: &[TokenType]) -> Result<Token> {
        if let Some(ret) = self.check_advance(types) {
            return ret;
        }
        Err(self.peek_err())
    }

    fn peek_err(&mut self) -> RloxError {
        match self.src.peek() {
            Some(Ok(ref token)) => Parser::unexpected(token),
            None => RloxError::Parse(0, "".to_string(), "Unexpectef EOF".to_string()),

            _ => self.src.next().unwrap().unwrap_err(),
        }
    }

    fn unexpected(token: &Token) -> RloxError {
        let lex = match token.token_type {
            TokenType::Eof => "EOF".to_string(),
            _ => token.lexeme.clone(),
        };
        RloxError::Parse(token.line, "Unexpected Token".to_string(), lex)
    }

    // This function is used in case of errors.
    // It swallows up the tokens on the current statement if there is a syntax error
    // and continues on the next statement
    fn synchronize(&mut self) {
        loop {
            if let Some(&Err(_)) = self.src.peek() {
                return;
            }

            let token = self.src.next();

            if token.is_none() {
                return;
            }

            if let Some(Ok(token)) = token {
                if token.token_type == SemiColon
                    && self.check(&[Class, Fun, Var, For, If, While, Print, Return])
                {
                    return;
                }
            }
        }
    }
}
