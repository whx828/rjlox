use super::error::Result;
use super::expr;
use super::expr::{Acceptor as ExprAcceptor, Expr};
use super::stmt;
use super::stmt::{Acceptor as StmtAcceptor, Stmt};
use crate::environment::Environment;
use crate::error::Error;
use crate::object::Object;
use crate::token::{Literal, Token, TokenType};
use log::error;
use std::rc::Rc;


pub struct Interpreter {
    env: Rc<Environment>,
}

impl Interpreter {
    pub fn new(env: Environment) -> Interpreter {
        Interpreter { env: Rc::new(env) }
    }

    pub fn interpret(&mut self, stmts: Vec<Stmt>) -> Result<()> {
        for stmt in stmts {
            match self.execute(&stmt) {
                Ok(_) => {}
                Err(r) => error!("{:?}", r),
            }
        }
        Ok(())
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<()> {
        stmt.accept(self)
    }

    fn execute_block(&mut self, stmts: &Vec<Stmt>, env: Environment) -> Result<()> {
        let previous_env = self.env.clone();
        self.env = Rc::new(env);

        for stmt in stmts {
            if let Err(e) = self.execute(stmt) {
                self.env = previous_env;
                return Err(e);
            }
        }

        self.env = previous_env;
        Ok(())
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Object> {
        expr.accept(self)
    }

    fn is_truthy(&self, object: Object) -> bool {
        match object {
            Object::Literal(literal) => match literal {
                Literal::Nil => false,
                Literal::Bool(b) => b,
                _ => true,
            },
        }
    }
}

impl expr::Visitor<Result<Object>> for Interpreter {
    fn visit_binary_expr(&mut self, left: &Expr, operator: &Token, right: &Expr) -> Result<Object> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;

        match (left, right) {
            (
                Object::Literal(Literal::Num(left_value)),
                Object::Literal(Literal::Num(right_value)),
            ) => match operator.token_type {
                TokenType::PLUS => {
                    let res = left_value + right_value;
                    Ok(Object::Literal(Literal::Num(res)))
                }
                TokenType::MINUS => {
                    let res = left_value - right_value;
                    Ok(Object::Literal(Literal::Num(res)))
                }
                TokenType::SLASH => {
                    let res = left_value / right_value;
                    Ok(Object::Literal(Literal::Num(res)))
                }
                TokenType::STAR => {
                    let res = left_value * right_value;
                    Ok(Object::Literal(Literal::Num(res)))
                }
                TokenType::GREATER => {
                    let res = left_value > right_value;
                    Ok(Object::Literal(Literal::Bool(res)))
                }
                TokenType::GreaterEqual => {
                    let res = left_value >= right_value;
                    Ok(Object::Literal(Literal::Bool(res)))
                }
                TokenType::LESS => {
                    let res = left_value < right_value;
                    Ok(Object::Literal(Literal::Bool(res)))
                }
                TokenType::LessEqual => {
                    let res = left_value <= right_value;
                    Ok(Object::Literal(Literal::Bool(res)))
                }
                TokenType::EqualEqual => {
                    let res = left_value == right_value;
                    Ok(Object::Literal(Literal::Bool(res)))
                }
                TokenType::BangEqual => {
                    let res = left_value != right_value;
                    Ok(Object::Literal(Literal::Bool(res)))
                }
                _ => Ok(Object::Literal(Literal::Nil)),
            },
            (
                Object::Literal(Literal::Str(left_value)),
                Object::Literal(Literal::Str(right_value)),
            ) => match operator.token_type {
                TokenType::PLUS => {
                    let mut res = left_value;
                    res.push_str(&right_value);
                    Ok(Object::Literal(Literal::Str(res)))
                }
                TokenType::EqualEqual => {
                    let res = left_value == right_value;
                    Ok(Object::Literal(Literal::Bool(res)))
                }
                TokenType::BangEqual => {
                    let res = left_value != right_value;
                    Ok(Object::Literal(Literal::Bool(res)))
                }
                _ => Ok(Object::Literal(Literal::Nil)),
            },
            (_, _) => match operator.token_type {
                TokenType::PLUS => Err(Error::RuntimeError(
                    operator.clone(),
                    String::from("Operands must be two numbers or two strings."),
                )),
                _ => Err(Error::RuntimeError(
                    operator.clone(),
                    String::from("Operands must be numbers."),
                )),
            },
        }
    }

    fn visit_grouping_expr(&mut self, expression: &Expr) -> Result<Object> {
        self.evaluate(expression)
    }

    fn visit_literal_expr(&mut self, expr: &Literal) -> Result<Object> {
        Ok(Object::Literal(expr.clone()))
    }

    fn visit_unary_expr(&mut self, operator: &Token, right: &Expr) -> Result<Object> {
        let right = self.evaluate(right)?;

        match operator.token_type {
            TokenType::MINUS => match right {
                Object::Literal(Literal::Num(x)) => {
                    let neg = -x;
                    Ok(Object::Literal(Literal::Num(neg)))
                }
                _ => Err(Error::RuntimeError(
                    operator.clone(),
                    String::from("Operand must be a number."),
                )),
            },
            TokenType::BANG => Ok(Object::Literal(Literal::Bool(!self.is_truthy(right)))),
            _ => Err(Error::RuntimeError(
                operator.clone(),
                String::from("Operand must be a number."),
            )),
        }
    }

    fn visit_var_expr(&mut self, name: &Token) -> Result<Object> { // 变量表达式
        self.env.get(name)
    }

    fn visit_assign_expr(&mut self, name: &Token, value: &Expr) -> Result<Object> {
        let value = self.evaluate(value)?;
        self.env.assign(name, &value)?;

        Ok(value)
    }

    fn visit_logic_expr(&mut self, left: &Expr, operator: &Token, right: &Expr) -> Result<Object> {
        let evaluated_left = self.evaluate(left);
        let is_left_truthy = self.is_truthy(evaluated_left.clone()?);

        if operator.token_type == TokenType::OR {
            if is_left_truthy {
                return evaluated_left;
            }
        } else if !is_left_truthy {
            return evaluated_left;
        }

        self.evaluate(right)
    }
}

impl stmt::Visitor<Result<()>> for Interpreter {
    fn visit_expression_stmt(&mut self, expression: &Expr) -> Result<()> {
        self.evaluate(expression)?;

        Ok(())
    }

    fn visit_print_stmt(&mut self, expression: &Expr) -> Result<()> {
        let value = self.evaluate(expression)?;
        println!("{value}");

        Ok(())
    }

    fn visit_var_stmt(&mut self, name: &Token, expression: &Expr) -> Result<()> {
        let value = self.evaluate(expression)?;
        self.env.define(name.lexeme.clone(), &value);

        Ok(())
    }

    fn visit_block_stmt(&mut self, stmts: &Vec<Stmt>) -> Result<()> {
        let env = self.env.clone();
        self.execute_block(stmts, Environment::new(Some(env)))?;

        Ok(())
    }

    fn visit_if_stmt(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: &Option<Box<Stmt>>,
    ) -> Result<()> {
        let evaluated = self.evaluate(condition)?;
        if self.is_truthy(evaluated) {
            self.execute(then_branch)?
        }

        match else_branch {
            Some(else_branch) => self.execute(else_branch)?,
            None => ()
        }

        Ok(())
    }

    fn visit_while_stmt(&mut self, condition: &Expr, body: &Stmt) -> Result<()> {
        loop {
            let evaluated_condition = self.evaluate(condition)?;
            if self.is_truthy(evaluated_condition) {
                self.execute(body)?
            } else {
                break;
            }
        }

        Ok(())
    }
}
