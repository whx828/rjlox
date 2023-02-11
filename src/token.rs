use std::fmt::{Debug, Formatter, Result};

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Eq, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Str(String),
    Num(f32),
    Bool(bool),
    Nil
}

#[derive(Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Literal,
    pub line: u32,
}

impl Debug for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if self.lexeme.is_empty() {
            write!(f, "{:?}", self.token_type)
        } else if self.literal == Literal::Nil {
            write!(f, "{:?} {}", self.token_type, self.lexeme)
        } else {
            write!(
                f,
                "{:?} {} {:?}",
                self.token_type, self.lexeme, self.literal
            )
        }
    }
}

impl Token {
    pub(crate) fn new(
        token_type: TokenType,
        lexeme: String,
        literal: Literal,
        line: u32,
    ) -> Self {
        Token {
            token_type,
            lexeme,
            literal,
            line,
        }
    }
}
