use crate::expr::Expr;
use crate::token::{Token, TokenType};
use crate::{token, Lox};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use crate::stmt::Stmt;

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

    // program → declaration* EOF ;
    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            let stmt = self.declaration();
            match stmt {
                Ok(stmt) => statements.push(stmt),
                Err(_e) => self.synchronize()
            }
        }

        statements
    }

    // declaration → varDecl | statement ; // 这样设计是因为不允许在块里声明语句
    fn declaration(&mut self) -> Result<Stmt, ParseError> {
        if self.match_one_token(&TokenType::VAR) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    // varDecl → "var" IDENTIFIER ( "=" expression )? ";" ;
    fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume(TokenType::IDENTIFIER, "Expect variable name.");

        match name {
            Ok(name) => {
                let mut initializer = Expr::Literal {
                    value: token::Literal::Nil,
                };

                if self.match_one_token(&TokenType::EQUAL) {

                    match self.expression() {
                        Ok(expr) => {
                            initializer = expr
                        }
                        Err(e) => return Err(e)
                    }
                }

                match self.consume(TokenType::SEMICOLON, "Expect ';' after variable declaration.") {
                    Ok(_) => Ok(Stmt::Var {
                        name,
                        expression: initializer
                    }),
                    Err(e) => Err(e)
                }
            }
            Err(e) => Err(e)
        }
    }

    // statement → exprStmt | printStmt | block ;
    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.match_one_token(&TokenType::PRINT) {
            return self.print_statement()
        }

        if self.match_one_token(&TokenType::LeftBrace) {
            return match self.block() {
                Ok(stmts) => Ok(Stmt::Block { stmts }),
                Err(e) => Err(e)
            }
        }

        self.expression_statement()
    }

    // printStmt → "print" expression ";" ;
    fn print_statement(&mut self) -> Result<Stmt, ParseError> {
        let value = self.expression().unwrap();

        match self.consume(TokenType::SEMICOLON, "Expect ';' after value.") {
            Ok(_) => Ok(Stmt::Print {expression: value}),
            Err(e) => Err(e),
        }
    }

    // exprStmt → expression ";" ;
    fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        match self.expression() {
            Ok(expr) => {
                match self.consume(TokenType::SEMICOLON, "Expect ';' after expression.") {
                    Ok(_) => Ok(Stmt::Expression {expression: expr}),
                    Err(e) => Err(e),
                }
            },
            Err(e) => Err(e)
        }
    }

    // block → "{" declaration* "}" ;
    fn block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            match self.declaration() {
                Ok(stmt) => statements.push(stmt),
                Err(e) => return Err(e)
            }
        }

        match self.consume(TokenType::RightBrace, "Expect '}' after block.") {
            Ok(_x) => Ok(statements),
            Err(e) => Err(e)
        }
    }

    // expression → assignment ;
    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    // assignment → IDENTIFIER "=" assignment | equality ; // 赋值是表达式而不是语句
    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.equality();

        if self.match_one_token(&TokenType::EQUAL) {
            let equals = self.previous();

            match self.assignment() {
                Ok(value) => {
                    match &expr {
                        Ok(expr) => {
                            match expr {
                                Expr::Variable { name} => {
                                    let name = name.to_owned();
                                    let value = Box::new(value);

                                    return Ok(Expr::Assign {
                                        name, value
                                    })
                                },
                                _ => {
                                    self.error(equals, "Invalid assignment target.");
                                }
                            }
                        },
                        Err(_e) => ()
                    }
                },
                Err(e) => return Err(e)
            }
        }

        expr
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

    // primary → NUMBER | STRING | "true" | "false" | "nil" | "(" expression ")" | IDENTIFIER ;
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

        if self.match_one_token(&TokenType::IDENTIFIER) {
            let value = self.previous();
            return Ok(Expr::Variable { name: value });
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
