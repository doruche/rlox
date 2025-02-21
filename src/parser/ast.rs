#![allow(unused)]

use crate::common::*;

#[derive(Debug)]
pub enum Expr {
    Literal(Value),
    UnaryExpr {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    BinaryExpr {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
}

#[derive(Debug)]
pub enum UnaryOp {
    Pos,
    Neg,
    Not,
}

#[derive(Debug)]
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