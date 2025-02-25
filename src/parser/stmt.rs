#![allow(unused)]

use std::fmt::format;

use crate::{common::*, error};
use crate::error::{Error, ErrorType};
use crate::lexer::TokenType;
use super::Expr::{self, *};
use super::parser::Parser;
use Stmt::*;

#[derive(Debug, Clone)]
pub enum Stmt {
    If {
        condition: Box<Expr>,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    While {
        condition: Box<Expr>,
        body: Vec<Stmt>,
        increment: Option<Box<Expr>>,
    },
    FunDecl {
        name: String,
        defined_line: usize,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    VarDecl {
        variable: String,
        defined_line: usize,
        initializer: Box<Expr>,  
    },
    Block(Vec<Stmt>),
    Expression(Box<Expr>),
    Print(Box<Expr>),
    Return {
        value: Box<Expr>,
        line: usize
    },
    Break(usize),
    Continue(usize),
    Empty,
}

impl Parser {
    pub(crate) fn declaration(&mut self) -> Result<Stmt, Error> {
        let stmt = match self.peek().kind {
            TokenType::Fun => self.fun_stmt("function"),
            TokenType::Var => self.var_stmt(),
            _ => self.statement(),
        };

        match stmt {
            s @Ok(_) => s,
            Err(e) => {
                self.has_error = true;
                self.synchronize();
                Err(e)
            }
        }
    }

    fn statement(&mut self) -> Result<Stmt, Error> {
        match self.peek().kind {
            TokenType::If => self.if_stmt(),
            TokenType::While => self.while_stmt(),
            TokenType::For => self.for_stmt(),
            TokenType::Return => self.return_stmt(),
            TokenType::Break => Ok(Stmt::Break(self.eat_current().line)),
            TokenType::Continue => Ok(Stmt::Continue(self.eat_current().line)),
            TokenType::Print => self.print_stmt(),
            TokenType::LBrace => Ok(Stmt::Block(self.block()?)),
            TokenType::SemiColon => Ok(self.empty()),
            _ => self.expr_stmt(),
        }
    }

    fn empty(&mut self) -> Stmt {
        self.eat_current();
        Empty
    }

    fn for_stmt(&mut self) -> Result<Stmt, Error> {
        self.eat_current();

        self.eat(&TokenType::LParen, "Expect '(' after 'for'.".to_string())?;
        let initializer = match self.peek().kind {
            TokenType::SemiColon => self.empty(),
            TokenType::Var => self.var_stmt()?,
            _ => self.expr_stmt()?,
        };

        let condition = Box::new(match self.peek().kind {
            TokenType::SemiColon => Literal(Value::Boolean(true)),
            _ => self.expr()?,
        });
        self.eat(&TokenType::SemiColon, "Expect ';' after the loop condition.".to_string())?;

        let increment = match self.peek().kind {
            TokenType::Rparen => None,
            _ => Some(Box::new(self.expr()?)),
        };
        self.eat(&TokenType::Rparen, "Expect ')' after clauses.".to_string())?;
        
        self.eat(&TokenType::LBrace, "Need a '{' to begin a for block.".to_string())?;
        let mut body = vec![];
        while !self.at_end()
        && self.peek().kind != TokenType::RBrace {
            body.push(self.declaration()?);
        }
        self.eat(&TokenType::RBrace, "Expect '}' after blocks.".to_string())?;       

        if increment.is_some() {
            body.push(Expression(increment.clone().unwrap()));
        }

        let while_body = While {
            condition, 
            body,
            increment,
        };

        Ok(Block(vec![initializer, while_body]))
    }

    fn while_stmt(&mut self) -> Result<Stmt, Error> {
        self.eat_current();
        let condition = Box::new(self.expr()?);

        self.eat(&TokenType::LBrace, "Need a '{' to begin a while block.".to_string())?;
        let mut body = vec![];
        while !self.at_end()
        && self.peek().kind != TokenType::RBrace {
            body.push(self.declaration()?);
        }
        self.eat(&TokenType::RBrace, "Expect '}' after blocks.".to_string())?;

        Ok(While {
            condition,
            body,
            increment: None,
        })
    }

    fn if_stmt(&mut self) -> Result<Stmt, Error> {
        self.eat_current();
        let condition = Box::new(self.expr()?);

        self.eat(&TokenType::LBrace, "Need a '{' to begin an if branch.".to_string())?;

        let mut then_branch = vec![];
        let mut else_branch = None;
        while self.peek().kind != TokenType::RBrace
        && !self.at_end() {
            then_branch.push(self.declaration()?);
        }
        self.eat(&TokenType::RBrace, "Expect '}' after blocks .".to_string())?;

        if self.peek().kind == TokenType::Else {
            let mut s = vec![];
            self.eat_current();
            if self.peek().kind == TokenType::If {
                else_branch = Some(vec![self.if_stmt()?]);
            }
            else {
                self.eat(&TokenType::LBrace, "Need a '{' to begin an else branch.".to_string())?;
                while self.peek().kind != TokenType::RBrace
                && !self.at_end() {
                    s.push(self.declaration()?);
                }
                self.eat(&TokenType::RBrace, "Expect '}' after blocks.".to_string())?;
                else_branch = Some(s);
            }
        }

        Ok(If { 
            condition, 
            then_branch, 
            else_branch,
        })
    }

    fn var_stmt(&mut self) -> Result<Stmt, Error> {
        self.eat_current();
        let token = self.eat(&TokenType::Identifier, "Expect a variable name.".to_string())?;

        let mut initializer = Box::new(Literal(Value::Nil));
        if self.peek().kind == TokenType::Equal {
            self.eat_current();
            initializer = Box::new(self.expr()?);
        }
        self.eat(&TokenType::SemiColon, "Expect ';' at the end".to_string())?;

        Ok(VarDecl {
            variable: match token.lexeme.unwrap() {
                Value::String(s) => s,
                _ => unreachable!(),
            },
            defined_line: token.line,
            initializer,
        })
    }

    fn block(&mut self) -> Result<Vec<Stmt>, Error> {
        self.eat_current();

        let mut statements = vec![];
        while self.peek().kind != TokenType::RBrace
        && !self.at_end() {
            statements.push(self.declaration()?);
        }

        self.eat(&TokenType::RBrace, "Expect '}' after blocks.".to_string())?;
        
        Ok(statements)
    } 

    fn print_stmt(&mut self) -> Result<Stmt, Error> {
        self.eat_current();
        let expr = Box::new(self.expr()?);
        self.eat(&TokenType::SemiColon, "Expected ';' at the end.".to_string())?;
        Ok(Print(expr))
    }

    fn expr_stmt(&mut self) -> Result<Stmt, Error> {
        let expr = Box::new(self.expr()?);
        self.eat(&TokenType::SemiColon, "Expected ';' at the end.".to_string())?;
        Ok(Expression(expr))
    }
}


// function and method
impl Parser {
  
    fn fun_stmt(&mut self, kind: &'static str) -> Result<Stmt, Error> {
        self.eat_current();
        let token = self.eat(&TokenType::Identifier, format!("Expect {kind} name."))?;
        self.eat(&TokenType::LParen, "Need a '(' to begin the parameter list.".to_string())?;

        let mut params = vec![];
        if self.peek().kind != TokenType::Rparen {
            if params.len() >= 255 {
                error::report(Error::new(ErrorType::ParserError, 
                    "Can't have more than 255 arguments.".to_string(), self.peek().line));
                self.has_error = true;
            }
            let param = self.eat(&TokenType::Identifier, "Expect parameter name.".to_string())?;
            let param = param.lexeme.unwrap().stringfy();
            params.push(param);
            while self.peek().kind != TokenType::Rparen {
                self.eat(&TokenType::Comma, "Expect ',' after parameters.".to_string())?;
                let param = self.eat(&TokenType::Identifier, "Expect parameter name.".to_string())?;
                let param = param.lexeme.unwrap().stringfy();
                params.push(param);
            }
        };
        self.eat(&TokenType::Rparen, "Expect ')' after parameters.".to_string())?;

        let body = self.block()?;

        Ok(FunDecl { 
            name: token.lexeme.unwrap().stringfy(), 
            defined_line: token.line, 
            params, 
            body, 
        })
    }

    fn return_stmt(&mut self) -> Result<Stmt, Error> {
        let token =self.eat_current();
        let mut value = Box::new(Literal(Value::Nil));
        if self.peek().kind != TokenType::SemiColon {
            value = Box::new(self.expr()?);
        }
        self.eat(&TokenType::SemiColon, "Expect ';' after return value.".to_string())?;
        
        Ok(Return {
            value,
            line: token.line,
        })
    }
}