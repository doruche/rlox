#![allow(unused)]

use std::{iter::Peekable, sync::Arc};

use crate::{common::*, error::{self, Error, ErrorType}};
use super::ast::*;
use super::ast::Expr::*;
use crate::lexer::{Lexer, TokenType, Token};

pub struct Parser {
    tokens: Vec<Token>,
    current_pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            current_pos: 0,
        }
    }

    pub fn parse(mut self) -> Result<Expr, ()> {
        self.conjuction()
    }

    fn conjuction(&mut self) -> Result<Expr, ()> {
        let mut exprs = vec![self.expr()?];
        
        while matches!(self.peek().kind,
        TokenType::Comma) {
            self.eat_current();
            exprs.push(self.expr()?);        
        }

        Ok(ConjunctionExpr { exprs })
    }

    fn expr(&mut self) -> Result<Expr, ()> {
        self.ternary()
    }


    fn ternary(&mut self) -> Result<Expr, ()> {
        let equa = self.equality()?;
        if self.peek().kind == TokenType::Question {
            let q = self.eat_current();
            let true_branch = Box::new(self.expr()?);
            self.eat(&TokenType::Colon, "Expected a ':'.".to_string());
            let false_branch = Box::new(self.expr()?);
            return Ok(TernaryExpr {
                condition: Box::new(equa),
                true_branch,
                false_branch,
                line: q.line,
            })
        } else {
            Ok(equa)
        }
    }

    fn equality(&mut self) -> Result<Expr, ()> {
        let mut equa = self.comparison()?;

        while matches!(self.peek().kind,
        TokenType::BangEqual | TokenType::EqualEqual) {
            let token = self.eat_current();
            let op = match token.kind {
                TokenType::BangEqual => BinaryOp::NotEq,
                TokenType::EqualEqual => BinaryOp::Eq,
                _ => unreachable!(),
            };
            equa = BinaryExpr {
                left: Box::new(equa),
                op,
                right: Box::new(self.comparison()?),
                line: token.line,
            };
        }

        Ok(equa)
    }

    fn comparison(&mut self) -> Result<Expr, ()> {
        let mut comp = self.term()?;

        while matches!(self.peek().kind,
        TokenType::Greater|TokenType::GreaterEqual|
        TokenType::Less|TokenType::LessEqual) {
            let token = self.eat_current();
            let op = match token.kind {
                TokenType::Greater => BinaryOp::Gt,
                TokenType::GreaterEqual => BinaryOp::GtEq,
                TokenType::Less => BinaryOp::Ls,
                TokenType::LessEqual => BinaryOp::LsEq,
                _ => unreachable!(),
            };
            comp = BinaryExpr {
                left: Box::new(comp),
                op,
                right: Box::new(self.term()?),
                line: token.line,
            };
        }

        Ok(comp)
    }

    fn term(&mut self) -> Result<Expr, ()> {
        let mut term = self.factor()?;

        while matches!(self.peek().kind,
        TokenType::Plus|TokenType::Minus) {
            let token = self.eat_current();
            let op = match token.kind {
                TokenType::Plus => BinaryOp::Add,
                TokenType::Minus => BinaryOp::Sub,
                _ => unreachable!(),
            };
            term = BinaryExpr {
                left: Box::new(term),
                op,
                right: Box::new(self.factor()?),
                line: token.line,
            };
        }

        Ok(term)
    }

    fn factor(&mut self) -> Result<Expr, ()> {
        let mut factor = self.unary()?;

        while matches!(self.peek().kind,
        TokenType::Star|TokenType::Slash) {
            let token = self.eat_current();
            let op = match token.kind {
                TokenType::Star => BinaryOp::Mul,
                TokenType::Slash => BinaryOp::Div,
                _ => unreachable!(),
            };
            factor = BinaryExpr {
                left: Box::new(factor),
                op,
                right: Box::new(self.unary()?),
                line: token.line,
            };
        }

        Ok(factor)
    }

    fn unary(&mut self) -> Result<Expr, ()> {
        if matches!(self.peek().kind,
        TokenType::Plus|TokenType::Minus|TokenType::Bang) {
            let token = self.eat_current();
            let op = match token.kind {
                TokenType::Plus => UnaryOp::Pos,
                TokenType::Minus => UnaryOp::Neg,
                TokenType::Bang => UnaryOp::Not,
                _ => unreachable!(),
            };
            return Ok(UnaryExpr {
                op,
                expr: Box::new(self.unary()?),
                line: token.line,
            });
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, ()> { 
        let token = self.eat_current();
        match token.kind {
            TokenType::Nil => Ok(Literal(Value::Nil)),
            TokenType::False => Ok(Literal(Value::Boolean(false))),
            TokenType::True => Ok(Literal(Value::Boolean(true))),
            TokenType::Number => Ok(Literal(token.lexeme.unwrap())),
            TokenType::String => Ok(Literal(token.lexeme.unwrap())),
            TokenType::LParen => {
                let e = self.expr();
                self.eat(&TokenType::Rparen, "Expect ')' after expression.".to_string())?;
                e
            }
            _ => {
                error::report(Error::new(ErrorType::ParserError, 
                format!("Expected an expression."), token.line));
                Err(())
            }
        }
    }

    fn eat(&mut self, kind: &TokenType, msg: String) -> Result<Token, ()> {
        if self.peek().kind == *kind {
            Ok(self.eat_current())
        } else {
            error::report(Error::new(ErrorType::ParserError, 
                msg, self.peek().line));
            Err(())
        }
    }

    fn eat_current(&mut self) -> Token {
        if !self.at_end() {
            self.current_pos += 1;
        }
        self.previous()
    }

    fn previous(&mut self) -> Token {
        let cur = self.tokens.get_mut(self.current_pos - 1).unwrap();
        std::mem::replace(cur, Token::def(cur.line))
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.current_pos).unwrap()
    }

    fn at_end(&self) -> bool {
        self.peek().kind == TokenType::Eof
    }

    fn synchronize(&mut self) {
        use TokenType::*;
        let mut token = self.eat_current();

        while !self.at_end() {
            if token.kind == SemiColon {
                return;
            }

            if matches!(self.peek().kind,
            Class|Fun|Var|For|If|While|Print|Return) {
                return;
            }

            token = self.eat_current();
        }
    }
}