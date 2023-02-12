use std::collections::HashMap;
use crate::token::{Literal, Token};

#[derive(Debug, Clone)]
pub struct Environment {
    pub enclosing: Box<Option<Environment>>,
    values: HashMap<String, Literal>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            enclosing: Box::new(None),
            values: HashMap::new()
        }
    }

    pub fn new_env(env: &Environment) -> Environment {
        let new_env = env.to_owned();
        Environment {
            enclosing: Box::new(Some(new_env)),
            values: HashMap::new()
        }
    }

    pub fn get(&self, name: &Token) -> Option<&Literal> {
        if self.values.contains_key(&name.lexeme) {
            return self.values.get(&name.lexeme)
        }

        match self.enclosing.as_ref() {
            Some(enclosing) => enclosing.get(name),
            None => None
        }
    }

    pub fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
    }

    pub fn assign(&mut self, name: Token, value: &Literal) -> Option<Literal> {
        if self.values.contains_key(&name.lexeme) {
            return self.values.insert(name.lexeme, value.to_owned())
        }

        match self.enclosing.as_ref() {
            Some(enclosing) => enclosing.to_owned().assign(name,value),
            None => None
        }
    }
}
