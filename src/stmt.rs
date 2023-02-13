use crate::expr::Expr;
use crate::token::Token;


pub trait Visitor<T> {
    fn visit_expression_stmt(&mut self, expression: &Expr) -> T;
    fn visit_print_stmt(&mut self, expression: &Expr) -> T;
    fn visit_var_stmt(&mut self, name: &Token, expression: &Expr) -> T;
    fn visit_block_stmt(&mut self, stmts: &Vec<Stmt>) -> T;
    fn visit_if_stmt(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: &Option<Box<Stmt>>,
    ) -> T;
    fn visit_while_stmt(&mut self, condition: &Expr, body: &Stmt) -> T;
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
        stmts: Vec<Stmt>,
    },
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
}

impl<T> Acceptor<T> for Stmt {
    fn accept(&self, visitor: &mut dyn Visitor<T>) -> T {
        match self {
            Stmt::Expression { expression } => visitor.visit_expression_stmt(expression),
            Stmt::Print { expression } => visitor.visit_print_stmt(expression),
            Stmt::Var { name, expression } => visitor.visit_var_stmt(name, expression),
            Stmt::Block { stmts } => visitor.visit_block_stmt(stmts),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => visitor.visit_if_stmt(condition, then_branch, else_branch),
            Stmt::While { condition, body } => visitor.visit_while_stmt(condition, body),
        }
    }
}
