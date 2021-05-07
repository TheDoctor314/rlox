use crate::expr::Expr;

#[derive(Debug)]
pub(crate) enum Stmt {
    Expression(Expr),
    Print(Expr),
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
}

impl Stmt {
    pub fn accept<T>(&self, v: &mut dyn Visitor<T>) -> T {
        use Stmt::*;

        match self {
            Expression(ref expr) => v.visit_expr_stmt(self, expr),
            Print(ref expr) => v.visit_print(self, expr),
        }
    }
}
