use super::error::{Error, Result};
use super::object::Object;
use super::token::Token;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;


#[derive(Debug, Clone)]
pub struct Environment {
    enclosing: Option<Rc<Environment>>, // 一个父环境可以有多个子环境 -> Rc
    values: RefCell<HashMap<String, Object>>, // 父环境的变量键值对可以被子环境改变 -> RefCell
}

impl Environment {
    pub fn new(enclosing: Option<Rc<Environment>>) -> Environment {
        let values = HashMap::new();
        Environment {
            enclosing,
            values: RefCell::new(values),
        }
    }

    pub fn define(&self, name: String, value: &Object) {
        // 在当前环境下存储键值对
        self.values.borrow_mut().insert(name, value.clone());
    }

    pub fn get(&self, name: &Token) -> Result<Object> {
        match self.values.borrow_mut().get(&name.lexeme) {
            Some(r) => Ok(r.clone()), // 在当前环境下找到了对应的键值对
            None => match self.enclosing.clone() {
                // 到上一层环境中寻找
                Some(enclosing) => enclosing.get(name),
                None => Err(Error::RuntimeError(
                    name.clone(),
                    format!("Undefined variable '{}'.", &name.lexeme),
                )),
            },
        }
    }

    pub fn assign(&self, name: &Token, value: &Object) -> Result<()> {
        if self.values.borrow().contains_key(&name.lexeme) {
            // 如果该变量是在当前环境下定义的
            self.values
                .borrow_mut()
                .insert(name.lexeme.clone(), value.clone()); // 那么就在当前环境下更新它的键值对
            return Ok(());
        }

        if self.enclosing.clone().is_some() {
            // 是否有父环境
            // 没在当前环境下定义，可能在父环境中有定义，如果父环境也没有，那就递归
            self.enclosing.clone().unwrap().assign(name, value)?;
            return Ok(());
        }

        // 递归到最后（全局环境）也没有发现定义，那就是一个未定义错误
        Err(Error::RuntimeError(
            name.clone(),
            format!("Undefined variable '{}'.", &name.lexeme),
        ))
    }
}
