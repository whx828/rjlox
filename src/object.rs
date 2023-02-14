use super::callable::Callable;
use super::token::Literal;
use std::fmt;

#[derive(Debug, Clone)]
pub enum Object {
    Literal(Literal),
    Callable(Callable),
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Object::Literal(l) => write!(f, "{l}"),
            Object::Callable(c) => write!(f, "{c}"),
        }
    }
}
