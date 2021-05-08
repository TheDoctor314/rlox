use std::iter::Peekable;

use crate::{
    error::{Result, RloxError},
    expr::Expr,
    scanner::Scanner,
    stmt::Stmt,
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
}

impl<'a> Iterator for Parser<'a> {
    type Item = Result<Stmt>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.src.peek().is_none() || self.check_advance(&[Eof]).is_some() {
            return None;
        }

        let res = self.statement();
        if res.is_err() {
            self.synchronize();
        }

        Some(res)
    }
}

// Statement related methods
impl<'a> Parser<'a> {
    fn statement(&mut self) -> Result<Stmt> {
        let token = self.check_advance(&[Print, Var, LBrace, If]);
        if token.is_none() {
            return self.expr_statement();
        }

        let token = token.unwrap()?;

        match token.token_type {
            Print => self.print_statement(),
            Var => self.decl_statement(),
            LBrace => self.block_statement(),
            If => self.if_statement(),
            _ => unreachable!(),
        }
    }

    fn print_statement(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        self.must_advance(&[SemiColon])?;
        Ok(Stmt::Print(expr))
    }

    fn expr_statement(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        self.must_advance(&[SemiColon])?;
        Ok(Stmt::Expression(expr))
    }

    fn decl_statement(&mut self) -> Result<Stmt> {
        let id = self.must_advance(&[Ident])?;
        if self.check_advance(&[Equal]).is_none() {
            self.must_advance(&[SemiColon])?;
            return Ok(Stmt::Declaration(id, None));
        }

        let init_expr = self.expression()?;
        self.must_advance(&[SemiColon])?;

        Ok(Stmt::Declaration(id, Some(Box::new(init_expr))))
    }

    fn block_statement(&mut self) -> Result<Stmt> {
        let mut statements = Vec::new();

        while self.check_advance(&[RBrace]).is_none() && self.src.peek().is_some() {
            statements.push(self.statement()?);
        }

        Ok(Stmt::Block(statements))
    }

    fn if_statement(&mut self) -> Result<Stmt> {
        self.must_advance(&[LParen])?;
        let cond = self.expression()?;
        self.must_advance(&[RParen])?;

        let then_stmt = self.statement()?;

        match self.check_advance(&[Else]) {
            Some(Err(e)) => Err(e),
            Some(Ok(_)) => Ok(Stmt::If(cond, Box::new(then_stmt), Some(Box::new(self.statement()?)))),
            None => Ok(Stmt::If(cond, Box::new(then_stmt), None)),
        }
    }
}

// Expression related methods
impl<'a> Parser<'a> {
    fn expression(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr> {
        let expr = self.equality()?;

        if let Some(res) = self.check_advance(&[Equal]) {
            let equals = res?;

            match expr {
                Expr::Identifier(token) => {
                    return Ok(Expr::Assignment(token, Box::new(self.assignment()?)))
                }
                _ => return Err(Parser::unexpected(&equals)),
            }
        }

        Ok(expr)
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
        if let Some(Ok(token)) =
            self.check_advance(&[Nil, False, True, Number, StringLiteral, Ident])
        {
            return match token.token_type {
                Ident => Ok(Expr::Identifier(token)),
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
