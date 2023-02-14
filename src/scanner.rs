use super::error::lexer_error;
use super::token::{Literal, Token, TokenType};

use std::collections::HashMap;
use std::string::String;

use lazy_static::lazy_static;

lazy_static! {
    static ref KEYWORDS: HashMap<String, TokenType> = {
        let mut keywords = HashMap::new();
        keywords.insert(String::from("and"), TokenType::AND);
        keywords.insert(String::from("class"), TokenType::CLASS);
        keywords.insert(String::from("else"), TokenType::ELSE);
        keywords.insert(String::from("false"), TokenType::FALSE);
        keywords.insert(String::from("for"), TokenType::FOR);
        keywords.insert(String::from("fun"), TokenType::FUN);
        keywords.insert(String::from("if"), TokenType::IF);
        keywords.insert(String::from("nil"), TokenType::NIL);
        keywords.insert(String::from("or"), TokenType::OR);
        keywords.insert(String::from("print"), TokenType::PRINT);
        keywords.insert(String::from("return"), TokenType::RETURN);
        keywords.insert(String::from("super"), TokenType::SUPER);
        keywords.insert(String::from("this"), TokenType::THIS);
        keywords.insert(String::from("true"), TokenType::TRUE);
        keywords.insert(String::from("var"), TokenType::VAR);
        keywords.insert(String::from("while"), TokenType::WHILE);
        keywords
    };
}

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Scanner {
        Scanner {
            source,
            tokens: Vec::<Token>::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(Token::new(
            TokenType::EOF,
            "".to_string(),
            Literal::Nil,
            self.line,
        ));

        Vec::clone(&self.tokens)
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::COMMA),
            '.' => self.add_token(TokenType::DOT),
            '-' => self.add_token(TokenType::MINUS),
            '+' => self.add_token(TokenType::PLUS),
            ';' => self.add_token(TokenType::SEMICOLON),
            '*' => self.add_token(TokenType::STAR),
            '!' => {
                if self.match_char('=') {
                    self.add_token(TokenType::BangEqual)
                } else {
                    self.add_token(TokenType::BANG)
                }
            }
            '=' => {
                if self.match_char('=') {
                    self.add_token(TokenType::EqualEqual)
                } else {
                    self.add_token(TokenType::EQUAL)
                }
            }
            '<' => {
                if self.match_char('=') {
                    self.add_token(TokenType::LessEqual)
                } else {
                    self.add_token(TokenType::LESS)
                }
            }
            '>' => {
                if self.match_char('=') {
                    self.add_token(TokenType::GreaterEqual)
                } else {
                    self.add_token(TokenType::GREATER)
                }
            }
            '/' => {
                if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::SLASH);
                }
            }
            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,
            '"' => self.string(),
            'o' => {
                if self.match_char('r') {
                    self.add_token(TokenType::OR);
                } else {
                    self.identifier();
                }
            }
            _ => {
                if c.is_ascii_digit() {
                    self.number();
                } else if c.is_alphabetic() || c == '_' {
                    self.identifier();
                } else {
                    lexer_error(self.line, "Unexpected character.");
                }
            }
        }
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            lexer_error(self.line, "Unterminated string.");
            return;
        }

        self.advance();

        let slice = self.source.get(self.start + 1..self.current - 1).unwrap();
        let value = String::from(slice);
        self.add_token_full(TokenType::STRING, Literal::Str(value));
    }

    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        // Double.parseDouble(source.substring(start, current))
        self.add_token_full(
            TokenType::NUMBER,
            Literal::Num(
                self.source
                    .get(self.start..self.current)
                    .unwrap()
                    .parse::<f32>()
                    .unwrap(),
            ),
        )
    }

    fn identifier(&mut self) {
        while self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
            self.advance();
        }

        let text = self.source.get(self.start..self.current).unwrap();
        let token_type_option = KEYWORDS.get(text);

        match token_type_option {
            Some(token_type) => {
                self.add_token(token_type.to_owned());
            }
            None => {
                self.add_token(TokenType::IDENTIFIER);
            }
        }
    }

    fn advance(&mut self) -> char {
        let char = self.source.chars().nth(self.current).unwrap();
        self.current += 1;
        char
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.add_token_full(token_type, Literal::Nil);
    }

    fn add_token_full(&mut self, token_type: TokenType, literal: Literal) {
        let a = self.source.get(self.start..self.current).unwrap();
        let text = String::from(a);
        self.tokens
            .push(Token::new(token_type, text, literal, self.line));
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.source.chars().nth(self.current).unwrap() != expected {
            return false;
        }

        self.current += 1;
        true
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\n';
        }
        self.source.chars().nth(self.current).unwrap()
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }

        self.source.chars().nth(self.current + 1).unwrap()
    }
}
