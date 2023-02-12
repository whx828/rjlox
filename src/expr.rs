use super::token;
use super::token::Token;

pub trait Visitor<T> {
    fn visit_binary_expr(&mut self, left: &Expr, operator: &Token, right: &Expr) -> T;
    fn visit_grouping_expr(&mut self, expression: &Expr) -> T;
    fn visit_literal_expr(&mut self, expr: &token::Literal) -> T;
    fn visit_unary_expr(&mut self, operator: &Token, right: &Expr) -> T;
    fn visit_var_expr(&mut self, name: &Token) -> T;
    fn visit_assign_expr(&mut self, name: &Token, value: &Expr) -> T;
}

pub trait Acceptor<T> {
    fn accept(&self, visitor: &mut dyn Visitor<T>) -> T;
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Assign {
        name: Token,
        value: Box<Expr>
    },
    Binary {
        left: Box<Expr>, // 注意自引用类型
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: token::Literal,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Variable {
        name: Token,
    }
}

impl<T> Acceptor<T> for Expr {
    fn accept(&self, visitor: &mut dyn Visitor<T>) -> T {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => visitor.visit_binary_expr(left, operator, right),
            Expr::Grouping { expression } => visitor.visit_grouping_expr(expression),
            Expr::Literal { value } => visitor.visit_literal_expr(value),
            Expr::Unary { operator, right } => visitor.visit_unary_expr(operator, right),
            Expr::Variable {name} => visitor.visit_var_expr(name),
            Expr::Assign {name, value} => visitor.visit_assign_expr(name, value)
        }
    }
}
