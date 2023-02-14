use super::error::Result;
use super::interpreter::Interpreter;
use super::object::Object;
use super::token::Literal;
use std::fmt;
use std::rc::Rc;

use crate::environment::Environment;
use crate::error::Error;
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
    closure: Rc<Environment>, // 闭包就是*函数定义*所在的作用域，函数在运行时并不知道自己是谁
}

impl Function {
    pub fn new(
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
        closure: Rc<Environment>,
    ) -> Function {
        Function {
            name,
            params,
            body,
            closure,
        }
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
                // 每个函数调用都有自己的环境来存储参数变量（运行时）
                let env = Environment::new(Some(function.closure.clone()));
                for i in 0..arguments.len() {
                    env.define(
                        function.params.get(i).cloned().unwrap().lexeme,
                        arguments.get(i).unwrap(),
                    );
                }

                // 函数调用时通过 Error::Return 判断遇到了 return 语句，立刻返回 return 的值
                match interpreter.execute_block(&function.body, env) {
                    Err(e) => {
                        return match e {
                            Error::Return(object) => Ok(object),
                            _ => Err(e),
                        }
                    }
                    _ => (),
                }

                Ok(Object::Literal(Literal::Nil))
            }
        }
    }
}
