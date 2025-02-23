#![allow(unused)]

use std::{iter::Peekable, ptr::eq, sync::Arc};

use crate::{common::*, error::{self, Error, ErrorType}};
use super::Expr::*;
use super::Stmt::*;
use super::*;
use crate::lexer::{Lexer, TokenType, Token};

pub struct Parser {
    tokens: Vec<Token>,
    current_pos: usize,

    has_error: bool,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            current_pos: 0,

            has_error: false,
        }
    }

    pub fn parse_prog(mut self) -> Result<Vec<Stmt>, ()> {
        let mut prog = vec![];
        while !self.at_end() {
            let stmt = self.declaration();
            if let Ok(stmt) = stmt {
                prog.push(stmt);
            }
        }
        if self.has_error {
            return Err(());
        }
        Ok(prog)
    }

    pub fn parse_expr(mut self) -> Result<Expr, ()> {
        self.expr()
    }

    fn declaration(&mut self) -> Result<Stmt, ()> {
        let stmt = if self.peek().kind == TokenType::Var {
            self.var_stmt()
        } else {
            self.statement()
        };
        if stmt.is_ok() {
            stmt
        } else {
            self.has_error = true;
            self.synchronize();
            Err(())
        }
    }

    fn statement(&mut self) -> Result<Stmt, ()> {
        match self.peek().kind {
            TokenType::If => self.if_stmt(),
            TokenType::While => self.while_stmt(),
            TokenType::For => self.for_stmt(),
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

    fn for_stmt(&mut self) -> Result<Stmt, ()> {
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
            _ => Some(self.expr()?),
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
            body.push(Expression(Box::new(increment.unwrap())));
        }

        let while_body = While {
            condition, 
            body, 
        };

        Ok(Block(vec![initializer, while_body]))
    }

    fn while_stmt(&mut self) -> Result<Stmt, ()> {
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
        })
    }

    fn if_stmt(&mut self) -> Result<Stmt, ()> {
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
            self.eat(&TokenType::LBrace, "Need a '{' to begin an else branch.".to_string())?;
            while self.peek().kind != TokenType::RBrace
            && !self.at_end() {
                s.push(self.declaration()?);
            }
            self.eat(&TokenType::RBrace, "Expect '}' after blocks.".to_string())?;
            else_branch = Some(s);
        }

        Ok(If { 
            condition, 
            then_branch, 
            else_branch,
        })
    }

    fn var_stmt(&mut self) -> Result<Stmt, ()> {
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

    fn block(&mut self) -> Result<Vec<Stmt>, ()> {
        self.eat_current();

        let mut statements = vec![];
        while self.peek().kind != TokenType::RBrace
        && !self.at_end() {
            statements.push(self.declaration()?);
        }

        self.eat(&TokenType::RBrace, "Expect '}' after blocks.".to_string())?;
        
        Ok(statements)
    } 

    fn print_stmt(&mut self) -> Result<Stmt, ()> {
        self.eat_current();
        let expr = Box::new(self.expr()?);
        self.eat(&TokenType::SemiColon, "Expected ';' at the end.".to_string())?;
        Ok(Print(expr))
    }

    fn expr_stmt(&mut self) -> Result<Stmt, ()> {
        let expr = Box::new(self.expr()?);
        self.eat(&TokenType::SemiColon, "Expected ';' at the end.".to_string())?;
        Ok(Expression(expr))
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
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ()> {
        let ternary = self.ternary()?;

        if self.peek().kind == TokenType::Equal {
            let equals = self.eat_current();
            let value = Box::new(self.assignment()?);

            if let Variable { name, refed_line } = ternary {
                return Ok(AssignExpr { 
                    name, 
                    refed_line, 
                    value, 
                });
            }

            error::report(Error::invalid_assign_error(equals.line));
            return Err(())
        }

        Ok(ternary)
    }


    fn ternary(&mut self) -> Result<Expr, ()> {
        let lor = self.logic_or()?;
        if self.peek().kind == TokenType::Question {
            let q = self.eat_current();
            let true_branch = Box::new(self.expr()?);
            self.eat(&TokenType::Colon, "Expected a ':'.".to_string())?;
            let false_branch = Box::new(self.expr()?);
            return Ok(TernaryExpr {
                condition: Box::new(lor),
                true_branch,
                false_branch,
                line: q.line,
            })
        } else {
            Ok(lor)
        }
    }

    fn logic_or(&mut self) -> Result<Expr, ()> {
        let mut lor = self.logic_and()?;

        while self.peek().kind == TokenType::Or {
            let or = self.eat_current();
            lor = BinaryExpr { 
                left: Box::new(lor), 
                op: BinaryOp::Or,
                right: Box::new(self.logic_and()?), 
                line: or.line 
            };
        }

        Ok(lor)
    }

    fn logic_and(&mut self) -> Result<Expr, ()> {
        let mut land = self.equality()?;
        
        while self.peek().kind == TokenType::And {
            let and = self.eat_current();
            land = BinaryExpr { 
                left: Box::new(land), 
                op: BinaryOp::And, 
                right: Box::new(self.equality()?), 
                line: and.line
            };
        }

        Ok(land)
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
            TokenType::Identifier => Ok(Variable {
                name: match token.lexeme.unwrap() {
                    Value::String(s) => s,
                    _ => unreachable!(),
                },
                refed_line: token.line,
            }),
            TokenType::LParen => {
                let expr = Box::new(self.expr()?);
                self.eat(&TokenType::Rparen, "Expect ')' after expression.".to_string())?;
                Ok(Expr::GroupExpr { expr })
            }
            _ => {
                error::report(Error::new(ErrorType::ParserError, 
                format!("Expected an expression."), token.line));
                self.has_error = true;
                Err(())
            }
        }
    }

    fn eat(&mut self, kind: &TokenType, msg: String) -> Result<Token, ()> {
        if self.peek().kind == *kind {
            Ok(self.eat_current())
        } else {
            error::report(Error::new(ErrorType::ParserError, 
                msg, self.previous().line));
            self.has_error = true;
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