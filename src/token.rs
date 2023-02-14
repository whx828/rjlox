use std::fmt;
use std::fmt::{Debug, Formatter, Result};
use std::hash::{Hash, Hasher};

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    COMMA,
    DOT,
    MINUS,
    PLUS,
    SEMICOLON,
    SLASH,
    STAR,

    // One or two character tokens.
    BANG,
    BangEqual,
    EQUAL,
    EqualEqual,
    GREATER,
    GreaterEqual,
    LESS,
    LessEqual,

    // Literals.
    IDENTIFIER,
    STRING,
    NUMBER,

    // Keywords.
    AND,
    CLASS,
    ELSE,
    FALSE,
    FUN,
    FOR,
    IF,
    NIL,
    OR,
    PRINT,
    RETURN,
    SUPER,
    THIS,
    TRUE,
    VAR,
    WHILE,

    EOF,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Str(String),
    Num(f32),
    Bool(bool),
    Nil,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Literal::Str(string) => write!(f, "{string}"),
            Literal::Num(num) => write!(f, "{num}"),
            Literal::Bool(bool) => write!(f, "{bool}"),
            Literal::Nil => write!(f, "nil"),
        }
    }
}

impl Eq for Literal {}

impl PartialEq for Literal {
    fn eq(&self, other: &Literal) -> bool {
        match (self, other) {
            (Literal::Bool(a), Literal::Bool(b)) => a.eq(b),
            (Literal::Str(a), Literal::Str(b)) => a.eq(b),
            (Literal::Num(a), Literal::Num(b)) => a.eq(b),
            (Literal::Nil, Literal::Nil) => true,
            (_, _) => false,
        }
    }
}

impl Hash for Literal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Literal::Str(s) => s.hash(state),
            Literal::Num(f) => f.to_bits().hash(state), // Rust 没有 f32/f64 hash 实现
            Literal::Bool(b) => b.hash(state),
            Literal::Nil => "".hash(state),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Literal,
    pub line: usize,
}

impl Debug for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if self.lexeme.is_empty() {
            write!(f, "{:?}", self.token_type)
        } else {
            write!(f, "{:?} {} {}", self.token_type, self.lexeme, self.literal)
        }
    }
}

impl Token {
    pub(crate) fn new(
        token_type: TokenType,
        lexeme: String,
        literal: Literal,
        line: usize,
    ) -> Self {
        Token {
            token_type,
            lexeme,
            literal,
            line,
        }
    }
}
