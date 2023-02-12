use super::expr;
use super::expr::{Acceptor as ExprAcceptor, Expr};
use super::stmt;
use super::stmt::{Acceptor as StmtAcceptor, Stmt};
use crate::token::{Literal, Token, TokenType};
use crate::{Lox};
use std::fmt::{Display, Formatter};
use crate::environment::Environment;

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
    env: &'int mut Environment,
}

impl Interpreter<'_> {
    pub fn new<'a>(lox: &'a mut Lox, env: &'a mut Environment) -> Interpreter<'a> {
        Interpreter {
            lox,
            env
        }
    }

    pub fn interpret(&mut self, stmts: Vec<Stmt>) {
        for stmt in stmts {
            if let Err(e) = self.execute(&stmt) {
                self.lox.runtime_error(e);
            }
        }
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        stmt.accept(self)
    }

    fn execute_block(&mut self, stmts: &Vec<Stmt>, env: Environment) -> Result<Box<Option<Environment>>, RuntimeError> {
        let previous_env = env.enclosing.clone();
        *self.env = env;

        for stmt in stmts {
            if let Err(e) = self.execute(stmt) { return Err(e) }
        }

        Ok(previous_env)
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Literal, RuntimeError> {
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

impl expr::Visitor<Result<Literal, RuntimeError>> for Interpreter<'_> {
    fn visit_binary_expr(
        &mut self,
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

    fn visit_grouping_expr(&mut self, expression: &Expr) -> Result<Literal, RuntimeError> {
        self.evaluate(expression)
    }

    fn visit_literal_expr(&mut self, expr: &Literal) -> Result<Literal, RuntimeError> {
        Ok(expr.to_owned())
    }

    fn visit_unary_expr(&mut self, operator: &Token, right: &Expr) -> Result<Literal, RuntimeError> {
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

    fn visit_var_expr(&mut self, name: &Token) -> Result<Literal, RuntimeError> {
        match self.env.get(name) {
            Some(x) => Ok(x.to_owned()),
            None => {
                let message = format!("Undefined variable '{}'.", name.lexeme);
                Err(RuntimeError::new(name, &message))
            }
        }
    }

    fn visit_assign_expr(&mut self, name: &Token, value: &Expr) -> Result<Literal, RuntimeError> {
        match self.evaluate(value) {
            Ok(literal) => {
                match self.env.assign(name.to_owned(), &literal) {
                    Some(_) => Ok(literal.clone()),
                    None => {
                        let message = format!("Undefined variable '{}'.", name.lexeme);
                        Err(RuntimeError::new(name, &message))
                    }
                }
            },
            Err(e) => Err(e)
        }
    }
}

impl stmt::Visitor<Result<(), RuntimeError>> for Interpreter<'_> {
    fn visit_expression_stmt(&mut self, expression: &Expr) -> Result<(), RuntimeError> {
        match self.evaluate(expression) {
            Ok(_) => Ok(()),
            Err(e) => Err(e)
        }
    }

    fn visit_print_stmt(&mut self, expression: &Expr) -> Result<(), RuntimeError> {
        let value = self.evaluate(expression);

        match value {
            Ok(l) => {
                println!("{l:?}");
                Ok(())
            },
            Err(e) => Err(e)
        }
    }

    fn visit_var_stmt(&mut self, name: &Token, expression: &Expr) -> Result<(), RuntimeError> {
        let mut value = Literal::Nil;
        let null = Expr::Literal {
            value: Literal::Nil,
        };

        if *expression != null {
            match self.evaluate(expression) {
                Ok(l) => {
                    value = l;
                },
                Err(e) => return Err(e)
            }
        }

        self.env.define(name.lexeme.to_string(), value);

        Ok(())
    }

    fn visit_block_stmt(&mut self, stmts: &Vec<Stmt>) -> Result<(), RuntimeError> {
        let env = self.env.clone();
        *self.env = self.execute_block(stmts, Environment::new_env(&env)).unwrap().unwrap();

        Ok(())
    }
}
