#![allow(unused)]

use crate::common::*;

#[derive(Debug)]
pub enum Expr {
    Literal(Value),
    Variable {
        name: String,
        refed_line: usize,
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
    GroupExpr {
        expr: Box<Expr>,
    },
    AssignExpr {
        name: String,
        refed_line: usize,
        value: Box<Expr>,
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