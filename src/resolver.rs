use super::error::{Error, Result};
use super::expr::Expr;
use super::expr::{Acceptor as ExprAcceptor, Visitor as ExprVisitor};
use super::interpreter::Interpreter;
use super::stmt::{Acceptor as StmtAcceptor, Stmt, Visitor as StmtVisitor};
use super::token::Literal;
use super::token::Token;

use std::collections::HashMap;

#[derive(Debug, Clone, Eq, PartialEq)]
enum FunctionType {
    NONE,
    FUNCTION,
}

#[derive(Debug)]
pub struct Resolver<'res> {
    interpreter: &'res mut Interpreter,
    pub scopes: Vec<HashMap<String, bool>>, // 所有局部作用域，不包括全局
    current_function: FunctionType,
}

impl<'res> Resolver<'res> {
    pub fn new(interpreter: &'res mut Interpreter) -> Resolver {
        Resolver {
            interpreter,
            scopes: Vec::new(),
            current_function: FunctionType::NONE,
        }
    }

    pub fn resolve_statements(&mut self, statements: &Vec<Stmt>) -> Result<()> {
        for statement in statements {
            self.resolve_statement(statement)?;
        }

        Ok(())
    }

    fn resolve_statement(&mut self, stmt: &Stmt) -> Result<()> {
        stmt.accept(self)
    }

    fn resolve_expr(&mut self, expr: &Expr) -> Result<()> {
        expr.accept(self)
    }

    fn resolve_function(
        &mut self,
        _name: &Token,
        params: &Vec<Token>,
        body: &Vec<Stmt>,
        fun_type: FunctionType,
    ) -> Result<()> {
        let enclosing_function = self.current_function.clone();
        self.current_function = fun_type;

        // 为函数体创建一个新的作用域，然后为每个函数参数绑定变量
        self.begin_scope();
        for param in params {
            self.declare(param)?;
            self.define(param);
        }
        self.resolve_statements(body)?;
        self.end_scope();

        self.current_function = enclosing_function;

        Ok(())
    }

    fn resolve_local(&mut self, expr: Expr, name: &Token) -> Result<()> {
        for (nesting_layer, scope) in self.scopes.iter().enumerate().rev() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter
                    .resolve(expr, self.scopes.len() - 1 - nesting_layer);
                return Ok(());
            }
        }

        // 如果遍历了所有的块作用域而未找到变量，我们就假设它是全局的
        Ok(())
    }

    fn declare(&mut self, name: &Token) -> Result<()> {
        if self.scopes.is_empty() {
            return Ok(());
        }

        let scope = self.scopes.last_mut().unwrap();
        if scope.contains_key(&name.lexeme) {
            // 禁止在局部作用域中出现像 `var a = a;` 这样的语句
            return Err(Error::ResolveError(
                name.clone(),
                String::from("Already a variable with this name in this scope."),
            ));
        }

        // 该变量存在, 但 false 的含义是其"尚未准备好"──"未初始化"
        scope.insert(name.lexeme.clone(), false);

        Ok(())
    }

    fn define(&mut self, name: &Token) {
        if self.scopes.is_empty() {
            return;
        }

        let scope = self.scopes.last_mut().unwrap();
        scope.insert(name.lexeme.clone(), true); // 将其标记为已初始化可供使用
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }
}

impl<'a> ExprVisitor<Result<()>> for Resolver<'a> {
    fn visit_binary_expr(&mut self, left: &Expr, _operator: &Token, right: &Expr) -> Result<()> {
        self.resolve_expr(left)?;
        self.resolve_expr(right)?;

        Ok(())
    }

    fn visit_grouping_expr(&mut self, expression: &Expr) -> Result<()> {
        self.resolve_expr(expression)?;

        Ok(())
    }

    fn visit_literal_expr(&mut self, _expr: &Literal) -> Result<()> {
        Ok(())
    }

    fn visit_unary_expr(&mut self, _operator: &Token, right: &Expr) -> Result<()> {
        self.resolve_expr(right)?;

        Ok(())
    }

    fn visit_var_expr(&mut self, name: &Token) -> Result<()> {
        if !self.scopes.is_empty() {
            if let Some(scope) = self.scopes.iter().peekable().peek() {
                if let Some(var) = scope.get(&name.lexeme) {
                    if *var == false {
                        return Err(Error::ResolveError(
                            name.clone(),
                            String::from("Cannot read local variable in its own initializer."),
                        ));
                    }
                }
            }
        }

        let expr = Expr::Variable { name: name.clone() };
        self.resolve_local(expr, name)?;

        Ok(())
    }

    fn visit_assign_expr(&mut self, name: &Token, value: &Expr) -> Result<()> {
        self.resolve_expr(value)?;
        self.resolve_local(value.clone(), name)?;

        Ok(())
    }

    fn visit_logic_expr(&mut self, left: &Expr, _operator: &Token, right: &Expr) -> Result<()> {
        self.resolve_expr(left)?;
        self.resolve_expr(right)?;

        Ok(())
    }

    fn visit_call_expr(
        &mut self,
        callee: &Expr,
        _paren: &Token,
        arguments: &Vec<Expr>,
    ) -> Result<()> {
        self.resolve_expr(callee)?;
        for arg in arguments {
            self.resolve_expr(arg)?;
        }

        Ok(())
    }
}

impl<'a> StmtVisitor<Result<()>> for Resolver<'a> {
    fn visit_expression_stmt(&mut self, expression: &Expr) -> Result<()> {
        self.resolve_expr(expression)?;

        Ok(())
    }

    fn visit_print_stmt(&mut self, expression: &Expr) -> Result<()> {
        self.resolve_expr(expression)?;

        Ok(())
    }

    fn visit_var_stmt(&mut self, name: &Token, expression: &Expr) -> Result<()> {
        self.declare(name)?;
        match expression {
            Expr::Literal {
                value: Literal::Nil,
            } => (),
            _ => self.resolve_expr(expression)?,
        }
        self.define(name);

        Ok(())
    }

    fn visit_block_stmt(&mut self, stmts: &Vec<Stmt>) -> Result<()> {
        self.begin_scope();
        self.resolve_statements(stmts)?;
        self.end_scope();

        Ok(())
    }

    fn visit_if_stmt(
        &mut self,
        condition: &Expr,
        then_branch: &Stmt,
        else_branch: &Option<Box<Stmt>>,
    ) -> Result<()> {
        self.resolve_expr(condition)?;
        self.resolve_statement(then_branch)?;
        match else_branch {
            Some(eb) => self.resolve_statement(eb),
            _ => Ok(()),
        }
    }

    fn visit_while_stmt(&mut self, condition: &Expr, body: &Stmt) -> Result<()> {
        self.resolve_expr(condition)?;
        self.resolve_statement(body)?;

        Ok(())
    }

    fn visit_fun_stmt(
        &mut self,
        name: &Token,
        params: &Vec<Token>,
        body: &Vec<Stmt>,
    ) -> Result<()> { // 函数既绑定名称又引入作用域
        // 在当前作用域内声明和定义函数名称
        // 在解析函数体之前就定义了函数名称，这让函数可以在它自己的体内递归地引用自己
        self.declare(name)?;
        self.define(name);

        self.resolve_function(name, params, body, FunctionType::FUNCTION)?;
        Ok(())
    }

    fn visit_return_stmt(&mut self, keyword: &Token, value: &Expr) -> Result<()> {
        if self.current_function == FunctionType::NONE {
            return Err(Error::ResolveError(
                keyword.clone(),
                String::from("Can't return from top-level code."),
            ));
        }

        match value {
            Expr::Literal {
                value: Literal::Nil,
            } => Ok(()),
            _ => self.resolve_expr(value),
        }
    }
}
