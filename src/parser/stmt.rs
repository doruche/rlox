#![allow(unused)]

use crate::lexer::Token;

use super::Expr;

#[derive(Debug)]
pub enum Stmt {
    If {
        condition: Box<Expr>,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    While {
        condition: Box<Expr>,
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
    Empty,
}