#![allow(unused)]

use std::sync::Arc;

use crate::{common::*, error::{self, Error, ErrorType}};
use super::ast::*;
use super::ast::Expr::*;
use crate::lexer::{Lexer, TokenType, Token};

pub struct Parser {
    lexer: Lexer,
    current_token: Token,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        let mut parser = Parser {
            lexer,
            current_token: Token::new(TokenType::And, None, 0),
        };
        parser.current_token = parser.lexer.next_token().unwrap();
        parser
    }

    pub fn parse(mut self) -> Expr {
        self.expr()
    }

    fn expr(&mut self) -> Expr {
        self.equality()
    }

    fn equality(&mut self) -> Expr {
        self.align();
        let mut equa = self.comparison();

        while matches!(self.current_token.kind, 
        TokenType::BangEqual | TokenType::EqualEqual) {
            let op = match self.current_token.kind {
                TokenType::BangEqual => BinaryOp::NotEq,
                TokenType::EqualEqual => BinaryOp::Eq,
                _ => unreachable!(),
            };
            self.eat_current();
            equa = BinaryExpr {
                left: Box::new(equa),
                op,
                right: Box::new(self.comparison()),
            };
            self.align();
        }

        equa
    }

    fn comparison(&mut self) -> Expr {
        self.align();
        let mut comp = self.term();

        while matches!(self.current_token.kind,
        TokenType::Greater|TokenType::GreaterEqual|
        TokenType::Less|TokenType::LessEqual) {
            let op = match self.current_token.kind {
                TokenType::Greater => BinaryOp::Gt,
                TokenType::GreaterEqual => BinaryOp::GtEq,
                TokenType::Less => BinaryOp::Ls,
                TokenType::LessEqual => BinaryOp::LsEq,
                _ => unreachable!(),
            };
            self.eat_current();
            comp = BinaryExpr {
                left: Box::new(comp),
                op,
                right: Box::new(self.term()),
            };
            self.align();
        }

        comp
    }

    fn term(&mut self) -> Expr {
        self.align();
        let mut term = self.factor();

        println!("in term {:?}", &self.current_token);
        while matches!(self.current_token.kind,
        TokenType::Plus|TokenType::Minus) {
            let op = match self.current_token.kind {
                TokenType::Plus => BinaryOp::Add,
                TokenType::Minus => BinaryOp::Sub,
                _ => unreachable!(),
            };
            self.eat_current();
            term = BinaryExpr {
                left: Box::new(term),
                op,
                right: Box::new(self.factor()),
            };
            self.align();
        }

        term
    }

    fn factor(&mut self) -> Expr {
        self.align();
        let mut factor = self.unary();

        while matches!(self.current_token.kind,
        TokenType::Star|TokenType::Slash) {
            let op = match self.current_token.kind {
                TokenType::Star => BinaryOp::Mul,
                TokenType::Slash => BinaryOp::Div,
                _ => unreachable!(),
            };
            self.eat_current();
            factor = BinaryExpr {
                left: Box::new(factor),
                op,
                right: Box::new(self.unary()),
            };
            self.align(); 
        }

        factor
    }

    fn unary(&mut self) -> Expr {
        self.align();
 
        println!("in unary {:?}", &self.current_token);
        if matches!(self.current_token.kind,
        TokenType::Plus|TokenType::Minus|TokenType::Bang) {
            let op = match self.current_token.kind {
                TokenType::Plus => UnaryOp::Pos,
                TokenType::Minus => UnaryOp::Neg,
                TokenType::Bang => UnaryOp::Not,
                _ => unreachable!(),
            };
            self.eat_current();
            return UnaryExpr {
                op,
                expr: Box::new(self.unary()),
            };
        }

        self.primary()
    }

    fn primary(&mut self) -> Expr {
        self.align();

        let token = self.eat_current();
        match token.kind {
            TokenType::False => Literal(Value::Boolean(false)),
            TokenType::True => Literal(Value::Boolean(true)),
            TokenType::Number => Literal(token.lexeme.unwrap()),
            TokenType::String => Literal(token.lexeme.unwrap()),
            TokenType::LParen => {
                let e = self.expr();
                self.eat(&TokenType::Rparen);
                e
            }
            _ => error::report_and_suspend(Error::new(ErrorType::ParserError, 
                format!("Unexpected token {token:?}"), token.line)),
        }
    }

    fn eat(&mut self, kind: &TokenType) {
        if &self.current_token.kind == kind {
            self.align();
            self.current_token = self.lexer.next_token().unwrap();
        } else {
            error::report_and_suspend(Error::new(ErrorType::ParserError, 
                format!("Expected {:?}, found {:?}", kind, &self.current_token), self.current_token.line));
        }
    }

    fn eat_current(&mut self) -> Token {
        self.align();
        let token = std::mem::replace(&mut self.current_token, Token::new(TokenType::And, None, 0));
        self.current_token = {
            match self.lexer.next_token() {
                Some(token) => token,
                None => {
                    self.align();
                    self.lexer.next_token().unwrap()
                },
            }
        };
        token
    }

    fn align(&mut self) {
        //println!("before align {:?}", &self.current_token);
        while self.lexer.peek_token().is_none() {
            self.lexer.next_token();
        }
        //println!("after align {:?}", &self.current_token);
    }

    fn at_end(&self) -> bool {
        self.current_token.kind == TokenType::Eof
    }
}