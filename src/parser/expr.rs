#![allow(unused)]

use crate::{common::*, error};
use crate::error::{Error, ErrorType};
use crate::lexer::TokenType;
use Expr::*;
use super::parser::Parser;
use super::Stmt;

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Value),
    Variable {
        name: String,
        refed_line: usize,
        resolve_distance: Option<usize>,
    },
    Call {
        callee: Box<Expr>,
        called_line: usize,
        arguments: Vec<Expr>,
    },
    Lambda {
        params: Vec<String>,
        body: Vec<Stmt>,
        defined_line: usize,
    },
    UnaryExpr {
        op: UnaryOp,
        expr: Box<Expr>,
        line: usize,
    },
    BinaryExpr {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
        line: usize,
    },
    TernaryExpr {
        condition: Box<Expr>,
        true_branch: Box<Expr>,
        false_branch: Box<Expr>,
        line: usize,
    },
    GroupExpr(Box<Expr>),
    AssignExpr {
        name: String,
        refed_line: usize,
        value: Box<Expr>,
        resolve_distance: Option<usize>,
    },
    ConjunctionExpr(Vec<Expr>),
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    Pos,
    Neg,
    Not,
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Gt,
    Ls,
    GtEq,
    LsEq,
    Eq,
    NotEq,
}

impl Parser {
    pub(crate) fn expr(&mut self) -> Result<Expr, Error> {
        self.conjuction()
    }

    fn conjuction(&mut self) -> Result<Expr, Error> {
        let mut exprs = vec![self.single_expr()?];
        
        while matches!(self.peek().kind,
        TokenType::Comma) {
            self.eat_current();
            exprs.push(self.single_expr()?);        
        }

        Ok(ConjunctionExpr(exprs))
    }

    fn single_expr(&mut self) -> Result<Expr, Error> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, Error> {
        let expr = self.lambda()?;

        if self.peek().kind == TokenType::Equal {
            let equals = self.eat_current();
            let value = Box::new(self.assignment()?);

            if let Variable { name, refed_line, ..} = expr {
                return Ok(AssignExpr { 
                    name, 
                    refed_line, 
                    value,
                    resolve_distance: None,
                });
            }

            return Err(Error::invalid_assign_error(equals.line))
        }

        Ok(expr)
    }

    fn lambda(&mut self) -> Result<Expr, Error> {
        if self.peek().kind == TokenType::Pipe {
            let pipe = self.eat_current();
            
            let mut params = vec![];
            if self.peek().kind != TokenType::Pipe {
                let param = self.eat(&TokenType::Identifier, "Expect parameter name.".to_string())?;
                let param = param.lexeme.unwrap().stringfy();
                params.push(param);
            
                while self.peek().kind != TokenType::Pipe {
                    if params.len() >= 16 {
                        error::report(Error::new(ErrorType::ParserError, 
                            "Can't have more than 16 arguments.".to_string(), self.peek().line));
                        self.has_error = true;
                    }
                    self.eat(&TokenType::Comma, "Expect ',' after parameters.".to_string())?;
                    let param = self.eat(&TokenType::Identifier, "Expect parameter name.".to_string())?;
                    let param = param.lexeme.unwrap().stringfy();
                    params.push(param);
                }
            };
            self.eat(&TokenType::Pipe, "Expect '|' after parameters.".to_string())?;

            match self.peek().kind {
                TokenType::LBrace => {
                    let body = self.block()?;
                    return Ok(Lambda { 
                        params, 
                        body, 
                        defined_line: pipe.line 
                    });
                },
                _ => {
                    let value = match self.single_expr() {
                        Ok(expr) => expr,
                        Err(_) => return Err(Error::new(ErrorType::ParserError, 
                            "Expect block or expression as the body.".to_string(), pipe.line)),
                    };
                    return Ok(Lambda {
                        params,
                        body: vec![Stmt::Return { value: Box::new(value), line: self.previous().line }],
                        defined_line: pipe.line,
                    })
                }
            };
        }


        self.ternary()
    }

    fn ternary(&mut self) -> Result<Expr, Error> {
        let lor = self.logic_or()?;
        if self.peek().kind == TokenType::Question {
            let q = self.eat_current();
            let true_branch = Box::new(self.conjuction()?);
            self.eat(&TokenType::Colon, "Expected a ':'.".to_string())?;
            let false_branch = Box::new(self.conjuction()?);
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

    fn logic_or(&mut self) -> Result<Expr, Error> {
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

    fn logic_and(&mut self) -> Result<Expr, Error> {
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

    fn equality(&mut self) -> Result<Expr, Error> {
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

    fn comparison(&mut self) -> Result<Expr, Error> {
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

    fn term(&mut self) -> Result<Expr, Error> {
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

    fn factor(&mut self) -> Result<Expr, Error> {
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

    fn unary(&mut self) -> Result<Expr, Error> {
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

        self.call()
    }

    fn call(&mut self) -> Result<Expr, Error> {
        let mut cal = self.primary()?;

        loop {
            if self.peek().kind == TokenType::LParen {
                self.eat_current();
                cal = self.finish_call(cal)?;
            } else {
                break;
            }
        }

        Ok(cal)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, Error> {
        let mut arguments = vec![];
        
        if self.peek().kind != TokenType::Rparen {
            arguments.push(self.single_expr()?);
            while self.peek().kind == TokenType::Comma {
                self.eat_current();
                arguments.push(self.single_expr()?);
                if arguments.len() >= 255 {
                    error::report(Error::new(ErrorType::ParserError, 
                        "Can't have more than 255 arguments.".to_string(), self.peek().line));
                    self.has_error = true;
                }
            }
        }
        let rparen = self.eat(&TokenType::Rparen, "Expect ')' after arguments.".to_string())?;
        
        Ok(Call {
            callee: Box::new(callee),
            called_line: rparen.line,
            arguments,
        })
    }

    fn primary(&mut self) -> Result<Expr, Error> { 
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
                resolve_distance: None,
            }),
            TokenType::LParen => {
                let expr = Box::new(self.conjuction()?);
                self.eat(&TokenType::Rparen, "Expect ')' after expression.".to_string())?;
                Ok(Expr::GroupExpr(expr))
            }
            _ => {
                self.has_error = true;
                Err(Error::new(ErrorType::ParserError, 
                format!("Expected an expression."), token.line))
            }
        }
    }
}