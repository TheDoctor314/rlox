use std::iter::Peekable;

use crate::{
    error::{Result, RloxError},
    expr::Expr,
    scanner::Scanner,
    stmt::Stmt,
    tokens::{Literal, Token, TokenType},
};
use TokenType::*;

pub(crate) struct Parser<'a> {
    src: Peekable<Scanner<'a>>,
    loop_depth: usize,
}

impl<'a> Parser<'a> {
    pub fn new(src: Scanner<'a>) -> Self {
        Self {
            src: src.peekable(),
            loop_depth: 0,
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
        let token = self.check_advance(&[Print, Var, LBrace, If, While, For, Break, Fun]);
        if token.is_none() {
            return self.expr_statement();
        }

        let token = token.unwrap()?;

        match token.token_type {
            Print => self.print_statement(),
            Var => self.decl_statement(),
            LBrace => self.block_statement(),
            If => self.if_statement(),
            While => self.while_statement(),
            For => self.for_statement(),
            Break => self.break_statement(token),
            Fun => self.function(),
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
            Some(Ok(_)) => Ok(Stmt::If(
                cond,
                Box::new(then_stmt),
                Some(Box::new(self.statement()?)),
            )),
            None => Ok(Stmt::If(cond, Box::new(then_stmt), None)),
        }
    }

    fn while_statement(&mut self) -> Result<Stmt> {
        self.loop_depth += 1;
        self.must_advance(&[LParen])?;
        let cond = self.expression()?;
        self.must_advance(&[RParen])?;

        let body = self.statement()?;
        self.loop_depth -= 1;

        Ok(Stmt::While(cond, Box::new(body)))
    }

    fn for_statement(&mut self) -> Result<Stmt> {
        self.must_advance(&[LParen])?;

        let init = match self.check_advance(&[SemiColon, Var]) {
            None => Some(self.expr_statement()?),
            Some(t) => match t?.token_type {
                SemiColon => None,
                Var => Some(self.decl_statement()?),
                _ => unreachable!(),
            },
        };

        let cond = match self.check_advance(&[SemiColon]) {
            Some(t) => Expr::Literal(Token {
                token_type: True,
                lexeme: "true".to_string(),
                literal: Some(Literal::Boolean(true)),
                ..t?
            }),
            None => {
                let expr = self.expression()?;
                self.must_advance(&[SemiColon])?;
                expr
            }
        };

        let inc = match self.check_advance(&[RParen]) {
            None => {
                let expr = self.expression()?;
                self.must_advance(&[RParen])?;
                Some(Stmt::Expression(expr))
            }
            Some(_) => None,
        };

        let mut body = self.statement()?;
        if inc.is_some() {
            body = Stmt::Block(vec![body, inc.unwrap()]);
        }

        body = Stmt::While(cond, Box::new(body));

        if init.is_some() {
            body = Stmt::Block(vec![init.unwrap(), body]);
        }

        Ok(body)
    }

    fn break_statement(&mut self, token: Token) -> Result<Stmt> {
        if self.loop_depth > 0 {
            self.must_advance(&[SemiColon])?;
            return Ok(Stmt::Break(token));
        }

        Err(RloxError::Break(token.line))
    }

    fn function(&mut self) -> Result<Stmt> {
        use crate::stmt::FUNCTION_MAX_ARGS;
        let name = self.must_advance(&[Ident])?;
        self.must_advance(&[LParen])?;

        let mut params = Vec::new();
        if !self.check(&[RParen]) {
            loop {
                if params.len() >= FUNCTION_MAX_ARGS {
                    return Err(RloxError::Parse(
                        name.line,
                        format!("Cannot have more than {} parameters", FUNCTION_MAX_ARGS),
                        name.lexeme,
                    ));
                }

                params.push(self.must_advance(&[Ident])?);

                if self.check_advance(&[Comma]).is_none() {
                    break;
                }
            }
        }

        self.must_advance(&[RParen])?;
        self.must_advance(&[LBrace])?;

        Ok(Stmt::Function(
            name,
            params,
            Box::new(self.block_statement()?),
        ))
    }
}

// Expression related methods
impl<'a> Parser<'a> {
    fn expression(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr> {
        let expr = self.logical_or()?;

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

    fn logical_or(&mut self) -> Result<Expr> {
        let mut expr = self.logical_and()?;

        while let Some(op) = self.check_advance(&[Or]) {
            expr = Expr::Logical(Box::new(expr), op?, Box::new(self.logical_and()?));
        }

        Ok(expr)
    }

    fn logical_and(&mut self) -> Result<Expr> {
        let mut expr = self.equality()?;

        while let Some(op) = self.check_advance(&[And]) {
            expr = Expr::Logical(Box::new(expr), op?, Box::new(self.equality()?));
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

        self.call()
    }

    fn call(&mut self) -> Result<Expr> {
        let mut expr = self.primary()?;

        loop {
            expr = match self.check_advance(&[LParen]) {
                Some(Err(e)) => return Err(e),
                Some(Ok(_)) => self.finish_call(expr)?,
                None => break,
            };
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr> {
        let mut args = Vec::new();
        if !self.check(&[RParen]) {
            loop {
                if args.len() >= crate::stmt::FUNCTION_MAX_ARGS {
                    return Err(RloxError::Parse(
                        0,
                        "Can't have more than 255 arguments".to_string(),
                        "".to_string(),
                    ));
                }
                args.push(self.expression()?);

                match self.check_advance(&[Comma]) {
                    Some(token) => token?,
                    None => break,
                };
            }
        }

        Ok(Expr::Call(
            Box::new(callee),
            self.must_advance(&[RParen])?,
            args,
        ))
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
