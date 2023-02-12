use crate::expr::{Acceptor, Expr, Visitor};
use crate::token::{Literal, Token, TokenType};
use crate::Lox;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct RuntimeError {
    pub token: Token,
    pub message: String,
}

impl RuntimeError {
    fn new(operator: &Token, message: &str) -> Self {
        RuntimeError {
            token: operator.to_owned(),
            message: message.to_string(),
        }
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "RuntimeError: {}", self.message)
    }
}

pub struct Interpreter<'int> {
    lox: &'int mut Lox,
}

impl Interpreter<'_> {
    pub fn new(lox: &mut Lox) -> Interpreter {
        Interpreter { lox }
    }

    pub fn interpret(&mut self, expr: Expr) -> Literal {
        match self.evaluate(&expr) {
            Ok(l) => {
                println!("{l:?}");
                l
            }
            Err(e) => {
                self.lox.runtime_error(e);
                Literal::Nil
            }
        }
    }

    fn evaluate(&self, expr: &Expr) -> Result<Literal, RuntimeError> {
        expr.accept(self)
    }

    fn is_truthy(&self, literal: Literal) -> bool {
        match literal {
            Literal::Nil => false,
            Literal::Bool(b) => b,
            _ => true,
        }
    }
}

impl Visitor<Result<Literal, RuntimeError>> for Interpreter<'_> {
    fn visit_binary_expr(
        &self,
        left: &Expr,
        operator: &Token,
        right: &Expr,
    ) -> Result<Literal, RuntimeError> {
        let left = self.evaluate(left).unwrap();
        let right = self.evaluate(right).unwrap();

        match (left, right) {
            (Literal::Num(left_value), Literal::Num(right_value)) => match operator.token_type {
                TokenType::PLUS => {
                    let res = left_value + right_value;
                    Ok(Literal::Num(res))
                }
                TokenType::MINUS => {
                    let res = left_value - right_value;
                    Ok(Literal::Num(res))
                }
                TokenType::SLASH => {
                    let res = left_value / right_value;
                    Ok(Literal::Num(res))
                }
                TokenType::STAR => {
                    let res = left_value * right_value;
                    Ok(Literal::Num(res))
                }
                TokenType::GREATER => {
                    let res = left_value > right_value;
                    Ok(Literal::Bool(res))
                }
                TokenType::GreaterEqual => {
                    let res = left_value >= right_value;
                    Ok(Literal::Bool(res))
                }
                TokenType::LESS => {
                    let res = left_value < right_value;
                    Ok(Literal::Bool(res))
                }
                TokenType::LessEqual => {
                    let res = left_value <= right_value;
                    Ok(Literal::Bool(res))
                }
                TokenType::EqualEqual => {
                    let res = left_value == right_value;
                    Ok(Literal::Bool(res))
                }
                TokenType::BangEqual => {
                    let res = left_value != right_value;
                    Ok(Literal::Bool(res))
                }
                _ => Ok(Literal::Nil),
            },
            (Literal::Str(left_value), Literal::Str(right_value)) => match operator.token_type {
                TokenType::PLUS => {
                    let mut res = left_value;
                    res.push_str(&right_value);
                    Ok(Literal::Str(res))
                }
                TokenType::EqualEqual => {
                    let res = left_value == right_value;
                    Ok(Literal::Bool(res))
                }
                TokenType::BangEqual => {
                    let res = left_value != right_value;
                    Ok(Literal::Bool(res))
                }
                _ => Ok(Literal::Nil),
            },
            (_, _) => match operator.token_type {
                TokenType::PLUS => Err(RuntimeError::new(
                    operator,
                    "Operands must be two numbers or two strings.",
                )),
                _ => Err(RuntimeError::new(operator, "Operands must be numbers.")),
            },
        }
    }

    fn visit_grouping_expr(&self, expression: &Expr) -> Result<Literal, RuntimeError> {
        self.evaluate(expression)
    }

    fn visit_literal_expr(&self, expr: &Literal) -> Result<Literal, RuntimeError> {
        Ok(expr.to_owned())
    }

    fn visit_unary_expr(&self, operator: &Token, right: &Expr) -> Result<Literal, RuntimeError> {
        let right = self.evaluate(right).unwrap();

        match operator.token_type {
            TokenType::MINUS => match right {
                Literal::Num(x) => {
                    let neg = -x;
                    Ok(Literal::Num(neg))
                }
                _ => Err(RuntimeError::new(operator, "Operand must be a number.")),
            },
            TokenType::BANG => Ok(Literal::Bool(!self.is_truthy(right))),
            _ => Err(RuntimeError::new(operator, "Operand must be a number.")),
        }
    }
}
