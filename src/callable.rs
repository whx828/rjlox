use super::error::Result;
use super::interpreter::Interpreter;
use super::object::Object;
use super::token::Literal;
use std::fmt;

use crate::environment::Environment;
use crate::stmt::Stmt;
use crate::token::Token;
use chrono::prelude::*;

pub(crate) trait LoxCallable {
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Object>) -> Result<Object>;
}

#[derive(Debug, Clone)]
pub enum Callable {
    Function(Function),
    Clock,
}

#[derive(Debug, Clone)]
pub struct Function {
    name: Token,
    params: Vec<Token>,
    body: Vec<Stmt>,
}

impl Function {
    pub fn new(name: Token, params: Vec<Token>, body: Vec<Stmt>) -> Function {
        Function { name, params, body }
    }
}

impl fmt::Display for Callable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Callable::Clock => write!(f, "<native fn>"),
            Callable::Function(function) => write!(f, "<fn {}>", function.name.lexeme),
        }
    }
}

impl LoxCallable for Callable {
    fn arity(&self) -> usize {
        match self {
            Callable::Clock => 0,
            Callable::Function(function) => function.params.len(),
        }
    }

    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Object>) -> Result<Object> {
        match self {
            Callable::Clock => {
                let now = Local::now().timestamp_millis() / 1000_i64;
                Ok(Object::Literal(Literal::Num(now as f32)))
            }
            Callable::Function(function) => {
                // 每个函数调用都有自己的环境来存储参数变量
                let env = Environment::new(Some(interpreter.globals.clone()));
                for i in 0..arguments.len() {
                    env.define(
                        function.params.get(i).cloned().unwrap().lexeme,
                        arguments.get(i).unwrap(),
                    );
                }

                interpreter.execute_block(&function.body, env)?;

                Ok(Object::Literal(Literal::Nil))
            }
        }
    }
}
