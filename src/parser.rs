use crate::expr::Expr;
use crate::token::{Token, TokenType};
use crate::{token, Lox};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub struct ParseError;

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ParseError is here!")
    }
}

impl Error for ParseError {}

pub struct Parser<'parser> {
    lox: &'parser mut Lox,
    tokens: Vec<Token>,
    current: u32,
}

impl Parser<'_> {
    pub fn new(lox: &mut Lox, tokens: Vec<Token>) -> Parser {
        Parser {
            lox,
            tokens,
            current: 0,
        }
    }

    pub fn parse(&mut self) -> Option<Expr> {
        match self.expression() {
            Ok(expression) => Some(expression),
            Err(_e) => None,
        }
    }

    // expression → equality ;
    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.equality()
    }

    // equality → comparison ( ( "!=" | "==" ) comparison )* ;
    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.comparison()?;

        // Rust 中没有可变参数列表，不得已使用 vec，但用切片应该是更好的选择
        let types = vec![TokenType::BangEqual, TokenType::EqualEqual];

        while self.match_token(&types) {
            let operator = self.previous();
            let right = self.comparison()?;

            left = Expr::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    // comparison → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.term()?;

        let types = vec![
            TokenType::GREATER,
            TokenType::GreaterEqual,
            TokenType::LESS,
            TokenType::LessEqual,
        ];

        while self.match_token(&types) {
            let operator = self.previous();
            let right = self.term()?;

            left = Expr::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            }
        }

        Ok(left)
    }

    // term → factor ( ( "-" | "+" ) factor )* ;
    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;

        let types = vec![TokenType::MINUS, TokenType::PLUS];

        while self.match_token(&types) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    // factor → unary ( ( "/" | "*" ) unary )* ;
    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;
        let types = vec![TokenType::SLASH, TokenType::STAR];

        while self.match_token(&types) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    // unary → ( "!" | "-" ) unary | primary ;
    fn unary(&mut self) -> Result<Expr, ParseError> {
        let types = vec![TokenType::BANG, TokenType::MINUS];

        if self.match_token(&types) {
            let operator = self.previous();
            let right = self.unary()?;

            return Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            });
        }

        self.primary()
    }

    // primary → NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" ;
    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.match_one_token(&TokenType::FALSE) {
            return Ok(Expr::Literal {
                value: token::Literal::Bool(false),
            });
        }

        if self.match_one_token(&TokenType::TRUE) {
            return Ok(Expr::Literal {
                value: token::Literal::Bool(true),
            });
        }

        if self.match_one_token(&TokenType::NIL) {
            return Ok(Expr::Literal {
                value: token::Literal::Nil,
            });
        }

        if self.match_one_token(&TokenType::STRING) {
            let value = self.previous().literal;
            return Ok(Expr::Literal { value });
        }

        if self.match_one_token(&TokenType::NUMBER) {
            let value = self.previous().literal;
            return Ok(Expr::Literal { value });
        }

        if self.match_one_token(&TokenType::LeftParen) {
            let expr = self.expression()?;

            return match self.consume(TokenType::RightParen, "Expect ')' after expression.") {
                Ok(_) => Ok(Expr::Grouping {
                    expression: Box::new(expr),
                }),
                Err(e) => Err(e),
            };
        }

        Err(self.error(self.peek(), "Expect expression."))
    }

    fn match_token(&mut self, types: &Vec<TokenType>) -> bool {
        for token_type in types {
            if self.match_one_token(token_type) {
                return true;
            }
        }

        false
    }

    fn match_one_token(&mut self, token_type: &TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token, ParseError> {
        if self.check(&token_type) {
            return Ok(self.advance());
        }

        Err(self.error(self.peek(), message))
    }

    fn error(&mut self, token: Token, message: &str) -> ParseError {
        self.lox.error_parse(token, message);
        ParseError {}
    }

    #[allow(dead_code)]
    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::SEMICOLON {
                return;
            }

            match self.peek().token_type {
                TokenType::CLASS
                | TokenType::FUN
                | TokenType::VAR
                | TokenType::FOR
                | TokenType::IF
                | TokenType::WHILE
                | TokenType::PRINT
                | TokenType::RETURN => return,
                _ => (),
            }

            self.advance();
        }
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek().token_type == token_type.clone()
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::EOF
    }

    fn peek(&self) -> Token {
        self.tokens.get(self.current as usize).unwrap().to_owned() // 为什么一定要 usize？
    }

    fn previous(&self) -> Token {
        self.tokens
            .get((self.current - 1) as usize)
            .unwrap()
            .to_owned()
    }
}
