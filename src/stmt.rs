use crate::expr::Expr;
use crate::token::Token;

pub trait Visitor<T> {
    fn visit_expression_stmt(&mut self, expression: &Expr) -> T;
    fn visit_print_stmt(&mut self, expression: &Expr) -> T;
    fn visit_var_stmt(&mut self, name: &Token, expression: &Expr) -> T;
    fn visit_block_stmt(&mut self, stmts: &Vec<Stmt>) -> T;
}

pub trait Acceptor<T> {
    fn accept(&self, visitor: &mut dyn Visitor<T>) -> T;
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression {
        expression: Expr,
    },
    Print {
        expression: Expr,
    },
    Var {
        name: Token,
        expression: Expr,
    },
    Block {
        stmts: Vec<Stmt>
    }
}

impl<T> Acceptor<T> for Stmt{
    fn accept(&self, visitor: &mut dyn Visitor<T>) -> T {
        match self {
            Stmt::Expression {expression} => {
                visitor.visit_expression_stmt(expression)
            },
            Stmt::Print {expression} => {
                visitor.visit_print_stmt(expression)
            },
            Stmt::Var {name, expression} => {
                visitor.visit_var_stmt(name, expression)
            },
            Stmt::Block {stmts} => {
                visitor.visit_block_stmt(stmts)
            }
        }
    }
}
