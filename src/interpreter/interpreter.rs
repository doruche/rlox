#![allow(unused)]

use crate::error::{Error, ErrorType};
use crate::parser::{Parser, Expr, BinaryOp, UnaryOp};
use crate::common::*;

pub struct Interpreter;

impl Interpreter {
    pub fn new() -> Self {
        Self
    }

    pub fn eval(&mut self, expr: &Expr) -> Result<Value, Error> {
        match expr {
            Expr::Literal(value) => Ok(value.clone()),
            Expr::UnaryExpr {
                op, 
                expr: uexpr,
                line,
            } => match op {
                UnaryOp::Pos => {
                    let v = self.eval(&uexpr)?;
                    if let Ok(v) = v.pos() {
                        Ok(v)
                    } else {
                        Err(Error::number_operator_error(*line))
                    }
                }, 
                UnaryOp::Neg => {
                    let v = self.eval(&uexpr)?;
                    if let Ok(v) = v.neg() {
                        Ok(v)
                    } else {
                        Err(Error::number_operator_error(*line))
                    }
                },
                UnaryOp::Not => Ok(self.eval(uexpr)?.not().ok().unwrap()),
            },
            Expr::BinaryExpr {
                left, 
                op, 
                right ,
                line,
            } => {
                let left = self.eval(&left)?;
                let right = self.eval(&right)?;
                match op {
                    BinaryOp::Add => {
                        if let Ok(v) = left.add(&right) {
                            Ok(v)
                        } else {
                            Err(Error::new(ErrorType::RuntimeError,
                                "Operands must be two numbers or strings".to_string(), *line))
                        }
                    },
                    BinaryOp::Sub => {
                        if let Ok(v) = left.sub(&right) {
                            Ok(v)
                        } else {
                            Err(Error::number_operator_error(*line))
                        }
                    },
                    BinaryOp::Mul => {
                        if let Ok(v) = left.mul(&right) {
                            Ok(v)
                        } else {
                            Err(Error::number_operator_error(*line))
                        }
                    },
                    BinaryOp::Div => {
                        let res = left.div(&right);
                        match res {
                            Err(true) => Err(Error::new(ErrorType::RuntimeError, 
                                "Divided by zero".to_string(), *line)),
                            Err(false) => Err(Error::number_operator_error(*line)),
                            Ok(v) => Ok(v),
                        }
                    },
                    BinaryOp::Gt => {
                        if let Ok(v) = left.gt(&right) {
                            Ok(v)
                        } else {
                            Err(Error::number_operator_error(*line))
                        }
                    }, 
                    BinaryOp::GtEq => {
                        if let Ok(v) = left.gteq(&right) {
                            Ok(v)
                        } else {
                            Err(Error::number_operator_error(*line))
                        }
                    },
                    BinaryOp::Ls => {
                        if let Ok(v) = left.ls(&right) {
                            Ok(v)
                        } else {
                            Err(Error::number_operator_error(*line))
                        }
                    },    
                    BinaryOp::LsEq => {
                        if let Ok(v) = left.gt(&right) {
                            Ok(v)
                        } else {
                            Err(Error::number_operator_error(*line))
                        }
                    },
                    BinaryOp::Eq => Ok(Value::Boolean(left == right)),
                    BinaryOp::NotEq => Ok(Value::Boolean(left != right)),
                    _ => todo!()
                }
            },
            Expr::TernaryExpr { 
                condition, 
                true_branch, 
                false_branch,
                ..
            } => {
                let condition = self.eval(&condition)?.is_truthy();
                if condition {
                    self.eval(&true_branch)
                } else {
                    self.eval(&false_branch)
                }
            },
            Expr::ConjunctionExpr { exprs } => {
                let mut res = Value::Nil;
                for cexpr in exprs {
                    res = self.eval(cexpr)?;
                }
                Ok(res)
            }
        }
    }
}