use crate::expr::Expr;
use crate::tokens::Token;

pub const FUNCTION_MAX_ARGS: usize = 255;

#[derive(Debug, Clone)]
pub(crate) enum Stmt {
    Expression(Expr),
    Print(Expr),
    Declaration(Token, Option<Box<Expr>>),
    Block(Vec<Stmt>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
    Break(Token),
    Function(Token, Vec<Token>, Box<Stmt>),
    Return(Token, Option<Box<Expr>>),
    Class(Token, Option<Box<Expr>>, Vec<Stmt>),
}

// Add more functions as variants are added to Stmt
pub(crate) trait Visitor<T> {
    fn visit_stmt(&mut self, _stmt: &Stmt) -> T {
        unimplemented!()
    }

    fn visit_expr_stmt(&mut self, _stmt: &Stmt, _expr: &Expr) -> T {
        self.visit_stmt(_stmt)
    }

    fn visit_print(&mut self, _stmt: &Stmt, _expr: &Expr) -> T {
        self.visit_stmt(_stmt)
    }

    fn visit_decl(&mut self, _stmt: &Stmt, _id: &Token, _init_expr: Option<&Expr>) -> T {
        self.visit_stmt(_stmt)
    }

    fn visit_block(&mut self, _stmt: &Stmt, _body: &[Stmt]) -> T {
        self.visit_stmt(_stmt)
    }

    fn visit_if(&mut self, _stmt: &Stmt, _cond: &Expr, _then: &Stmt, _else: Option<&Stmt>) -> T {
        self.visit_stmt(_stmt)
    }

    fn visit_while(&mut self, _stmt: &Stmt, _cond: &Expr, _body: &Stmt) -> T {
        self.visit_stmt(_stmt)
    }

    fn visit_break(&mut self, _stmt: &Stmt, _token: &Token) -> T {
        self.visit_stmt(_stmt)
    }

    fn visit_func(&mut self, _stmt: &Stmt, _name: &Token, _params: &[Token], _body: &Stmt) -> T {
        self.visit_stmt(_stmt)
    }

    fn visit_return(&mut self, _stmt: &Stmt, _keyword: &Token, _val: Option<&Expr>) -> T {
        self.visit_stmt(_stmt)
    }

    fn visit_class(
        &mut self,
        _stmt: &Stmt,
        _name: &Token,
        _parent: Option<&Expr>,
        _methods: &[Stmt],
    ) -> T {
        self.visit_stmt(_stmt)
    }
}

impl Stmt {
    pub fn accept<T>(&self, v: &mut dyn Visitor<T>) -> T {
        use Stmt::*;

        match self {
            Expression(ref expr) => v.visit_expr_stmt(self, expr),
            Print(ref expr) => v.visit_print(self, expr),
            Declaration(ref id, ref init) => {
                v.visit_decl(self, id, init.as_ref().map(|init| init.as_ref()))
            }
            Block(ref body) => v.visit_block(self, body),
            If(ref cond, ref then, ref else_stmt) => v.visit_if(
                self,
                cond,
                then.as_ref(),
                else_stmt.as_ref().map(|e| e.as_ref()),
            ),
            While(ref cond, ref body) => v.visit_while(self, cond, body),
            Break(ref token) => v.visit_break(self, token),
            Function(ref name, ref params, ref body) => v.visit_func(self, name, params, body),
            Return(ref token, ref val) => {
                v.visit_return(self, token, val.as_ref().map(|val| val.as_ref()))
            }
            Class(ref name, ref parent, ref methods) => {
                v.visit_class(self, name, parent.as_ref().map(|p| p.as_ref()), methods)
            }
        }
    }
}
