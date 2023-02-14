use super::object::Object;
use super::token::{Token, TokenType};

#[derive(Debug, Clone)]
pub enum Error {
    ParseError(String),
    RuntimeError(Token, String),
    Return(Object),
    ResolveError(Token, String),
}

pub type Result<T> = std::result::Result<T, Error>;

fn report(line: usize, place: &str, message: &str) {
    println!("[line {}] Error{}: {}", line, place, message);
}

pub fn parser_error(token: Token, message: &str) {
    if token.token_type == TokenType::EOF {
        report(token.line, " at end", message)
    } else {
        report(token.line, "", message)
    }
}

pub fn lexer_error(line: usize, message: &str) {
    report(line, "", message)
}
