#![allow(unused)]

use crate::common::*;

#[derive(Debug)]
pub enum Expr {
    Literal(Value),
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
    ConjunctionExpr {
        exprs: Vec<Expr>,
    },
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