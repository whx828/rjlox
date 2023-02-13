use crate::error;
use crate::error::Error;
use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::token::{Literal, Token, TokenType};


type ParseResult<T> = Result<T, Error>;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, current: 0 }
    }

    // program → declaration* EOF ;
    pub fn parse(&mut self) -> ParseResult<Vec<Stmt>> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        Ok(statements)
    }

    // declaration → varDecl | statement ; // 这样设计是因为不允许在块里声明语句
    fn declaration(&mut self) -> ParseResult<Stmt> {
        let result = if self.match_one_token(&TokenType::VAR) {
            self.var_declaration()
        } else {
            self.statement()
        };

        match result {
            Ok(r) => Ok(r),
            Err(e) => {
                self.synchronize();
                Err(e)
            }
        }
    }

    // varDecl → "var" IDENTIFIER ( "=" expression )? ";" ;
    fn var_declaration(&mut self) -> ParseResult<Stmt> {
        let name = self.consume(TokenType::IDENTIFIER, "Expect variable name.")?;

        let initializer = if self.match_one_token(&TokenType::EQUAL) {
            self.expression()?
        } else {
            Expr::Literal {
                value: Literal::Nil,
            }
        };

        self.consume(
            TokenType::SEMICOLON,
            "Expect ';' after variable declaration.",
        )?;

        Ok(Stmt::Var {
            name,
            expression: initializer,
        })
    }

    // statement → exprStmt | forStmt | ifStmt | printStmt | whileStmt | block ;
    fn statement(&mut self) -> ParseResult<Stmt> {
        if self.match_one_token(&TokenType::FOR) {
            return self.for_statement();
        }

        if self.match_one_token(&TokenType::IF) {
            return self.if_statement();
        }

        if self.match_one_token(&TokenType::PRINT) {
            return self.print_statement();
        }

        if self.match_one_token(&TokenType::WHILE) {
            return self.while_statement();
        }

        if self.match_one_token(&TokenType::LeftBrace) {
            return Ok(Stmt::Block {
                stmts: self.block()?,
            });
        }

        self.expression_statement()
    }

    // printStmt → "print" expression ";" ;
    fn print_statement(&mut self) -> ParseResult<Stmt> {
        let value = self.expression()?;
        self.consume(TokenType::SEMICOLON, "Expect ';' after value.")?;

        Ok(Stmt::Print { expression: value })
    }

    // exprStmt → expression ";" ;
    fn expression_statement(&mut self) -> ParseResult<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenType::SEMICOLON, "Expect ';' after expression.")?;

        Ok(Stmt::Expression { expression: expr })
    }

    // ifStmt → "if" "(" expression ")" statement ( "else" statement )? ;
    fn if_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after if condition.")?;

        let then_branch = self.statement()?;

        let mut else_branch = None;
        if self.match_one_token(&TokenType::ELSE) {
            else_branch = Some(Box::new(self.statement()?));
        }

        Ok(Stmt::If {
            condition,
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    // whileStmt → "while" "(" expression ")" statement ;
    fn while_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after condition.")?;
        let body = self.statement()?;

        Ok(Stmt::While {
            condition,
            body: Box::new(body),
        })
    }

    // forStmt → "for" "(" ( varDecl | exprStmt | ";" ) expression? ";" expression? ")" statement ;
    fn for_statement(&mut self) -> ParseResult<Stmt> {
        // todo
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.")?;

        let initializer = if self.match_one_token(&TokenType::SEMICOLON) {
            None
        } else if self.match_one_token(&TokenType::VAR) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if !self.check(&TokenType::SEMICOLON) {
            self.expression()?
        } else {
            Expr::Literal {
                value: Literal::Bool(true),
            }
        };
        self.consume(TokenType::SEMICOLON, "Expect ';' after loop condition.")?;

        let increment = if !self.check(&TokenType::RightParen) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenType::RightParen, "Expect ')' after for clauses.")?;

        let mut body = self.statement()?;
        if increment.is_some() {
            body = Stmt::Block {
                stmts: vec![
                    body,
                    Stmt::Expression {
                        expression: increment.unwrap(),
                    },
                ],
            }
        }
        body = Stmt::While {
            condition,
            body: Box::new(body),
        };
        if initializer.is_some() {
            body = Stmt::Block {
                stmts: vec![initializer.unwrap(), body],
            }
        }

        Ok(body)
    }

    // block → "{" declaration* "}" ;
    fn block(&mut self) -> ParseResult<Vec<Stmt>> {
        let mut statements = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(statements)
    }

    // expression → assignment ;
    fn expression(&mut self) -> ParseResult<Expr> {
        self.assignment()
    }

    // assignment → IDENTIFIER "=" assignment | logic_or ; // 赋值是表达式而不是语句
    fn assignment(&mut self) -> ParseResult<Expr> {
        let expr = self.or()?;

        if self.match_one_token(&TokenType::EQUAL) {
            let equals = self.previous();
            let value = self.assignment()?;

            return match expr {
                Expr::Variable { name } => Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                }),
                _ => Err(Self::error(equals, "Invalid assignment target.")),
            };
        }

        Ok(expr)
    }

    // logic_or → logic_and ( "or" logic_and )* ;
    fn or(&mut self) -> ParseResult<Expr> {
        let mut expr = self.and()?;

        while self.match_one_token(&TokenType::OR) {
            let operator = self.previous();
            let right = self.and()?;
            expr = Expr::Logic {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    // logic_and → equality ( "and" equality )* ;
    fn and(&mut self) -> ParseResult<Expr> {
        let mut expr = self.equality()?;
        while self.match_one_token(&TokenType::AND) {
            let operator = self.previous();
            let right = self.equality()?;
            expr = Expr::Logic {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    // equality → comparison ( ( "!=" | "==" ) comparison )* ;
    fn equality(&mut self) -> ParseResult<Expr> {
        let mut left = self.comparison()?;

        // Rust 中没有可变参数列表，不得已使用 vec，可能用切片是更好的选择
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
    fn comparison(&mut self) -> ParseResult<Expr> {
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
    fn term(&mut self) -> ParseResult<Expr> {
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
    fn factor(&mut self) -> ParseResult<Expr> {
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
    fn unary(&mut self) -> ParseResult<Expr> {
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
    fn primary(&mut self) -> ParseResult<Expr> {
        if self.match_one_token(&TokenType::FALSE) {
            return Ok(Expr::Literal {
                value: Literal::Bool(false),
            });
        }

        if self.match_one_token(&TokenType::TRUE) {
            return Ok(Expr::Literal {
                value: Literal::Bool(true),
            });
        }

        if self.match_one_token(&TokenType::NIL) {
            return Ok(Expr::Literal {
                value: Literal::Nil,
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

            self.consume(TokenType::RightParen, "Expect ')' after expression.")?;

            return Ok(Expr::Grouping {
                expression: Box::new(expr),
            });
        }

        Err(Self::error(self.peek(), "Expect expression."))
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

    fn consume(&mut self, token_type: TokenType, message: &str) -> ParseResult<Token> {
        if self.check(&token_type) {
            return Ok(self.advance());
        }

        Err(Self::error(self.peek(), message))
    }

    fn error(token: Token, message: &str) -> Error {
        error::parser_error(token, message);
        Error::ParseError(String::from(message))
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
        self.tokens.get(self.current).unwrap().to_owned()
    }

    fn previous(&self) -> Token {
        self.tokens.get(self.current - 1).unwrap().to_owned()
    }
}
