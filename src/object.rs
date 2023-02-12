use super::token::Literal;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Object {
    Literal(Literal),
}
