use super::token;
use super::token::Token;

pub trait Visitor<T> {
    fn visit_binary_expr(&self, left: &Expr, operator: &Token, right: &Expr) -> T;
    fn visit_grouping_expr(&self, expression: &Expr) -> T;
    fn visit_literal_expr(&self, expr: &token::Literal) -> T;
    fn visit_unary_expr(&self, operator: &Token, right: &Expr) -> T;
}

pub trait Acceptor<T> {
    fn accept(&self, visitor: &dyn Visitor<T>) -> T;
}

#[derive(Debug, Clone)]
pub enum Expr {
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
}

impl<T> Acceptor<T> for Expr {
    fn accept(&self, visitor: &dyn Visitor<T>) -> T {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => visitor.visit_binary_expr(left, operator, right),
            Expr::Grouping { expression } => visitor.visit_grouping_expr(expression),
            Expr::Literal { value } => visitor.visit_literal_expr(value),
            Expr::Unary { operator, right } => visitor.visit_unary_expr(operator, right),
        }
    }
}
