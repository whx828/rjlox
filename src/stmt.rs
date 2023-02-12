use crate::expr::Expr;


pub trait Visitor<T> {
    fn visit_expression_stmt(&self, expression: &Expr) -> T;
    fn visit_print_stmt(&self, expression: &Expr) -> T;
}

pub trait Acceptor<T> {
    fn accept(&self, visitor: &dyn Visitor<T>) -> T;
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression {
        expression: Expr,
    },
    Print {
        expression: Expr,
    },
}

impl<T> Acceptor<T> for Stmt{
    fn accept(&self, visitor: &dyn Visitor<T>) -> T {
        match self {
            Stmt::Expression {expression} => {
                visitor.visit_expression_stmt(expression)
            },
            Stmt::Print {expression} => {
                visitor.visit_print_stmt(expression)
            }
        }
    }
}
