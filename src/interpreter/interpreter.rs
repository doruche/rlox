#![allow(unused)]

use crate::error::{Error, ErrorType};
use crate::parser::{Parser, Expr, BinaryOp, UnaryOp, Stmt};
use crate::common::*;

use super::environment::Environment;

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
        }
    }

    pub fn exec(&mut self, statements: &Vec<Stmt>) -> Result<(), Error> {
        for stmt in statements {
            match stmt {
                Stmt::VarDecl { 
                    variable, 
                    defined_line, 
                    initializer 
                } => {
                    let value = self.eval(&initializer)?;
                    self.environment.define(variable.clone(), value);
                },
                Stmt::While {
                    condition, 
                    body 
                } => {
                    self.environment.enter_scope();
                    while self.eval(&condition)?.is_truthy() {
                        self.exec(body)?;
                    }
                    self.environment.exit_scope();
                },
                Stmt::If {
                    condition, 
                    then_branch, 
                    else_branch 
                } => {
                    self.environment.enter_scope();
                    if self.eval(&condition)?.is_truthy() {
                        self.exec(then_branch)?;
                    } else if else_branch.is_some() {
                        let else_branch = else_branch.as_ref().unwrap();
                        if else_branch.len() == 1 {
                            if let Stmt::If{..} = else_branch.first().unwrap() {
                                self.environment.exit_scope();
                            }
                        }
                        self.exec(else_branch)?;
                    }
                    self.environment.exit_scope();
                },
                Stmt::Expression(expr) => {
                    let _ = self.eval(expr)?;
                },
                Stmt::Print(expr) => {
                    let v = self.eval(expr)?;
                    println!("{}", v);
                },
                Stmt::Block(statements) => {
                    self.environment.enter_scope();
                    self.exec(statements)?;
                    self.environment.exit_scope();
                },
                Stmt::Empty => (),
            };
        }
        Ok(())
    }

    pub fn eval(&mut self, expr: &Expr) -> Result<Value, Error> {
        match expr {
            Expr::Literal(value) => Ok(value.clone()),
            Expr::Variable { name, refed_line } => Ok(self.environment.lookup(name, *refed_line)?.clone()),
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
                match op {
                    BinaryOp::And => {
                        if !left.is_truthy() {
                            return Ok(left);
                        }
                        return Ok(self.eval(&right)?);
                    },
                    BinaryOp::Or => {
                        if left.is_truthy() {
                            return Ok(left);
                        }
                        return Ok(self.eval(&right)?);
                    },
                    _ => (),
                };
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
                        if let Ok(v) = left.lseq(&right) {
                            Ok(v)
                        } else {
                            Err(Error::number_operator_error(*line))
                        }
                    },
                    BinaryOp::Eq => Ok(Value::Boolean(left == right)),
                    BinaryOp::NotEq => Ok(Value::Boolean(left != right)),
                    _ => unreachable!(),
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
            Expr::GroupExpr { expr } => self.eval(&expr),
            Expr::AssignExpr {
                name, 
                refed_line, 
                value 
            } => {
                let value = self.eval(&value)?;
                let variable = self.environment.lookup_mut(name, *refed_line)?;
                *variable = value.clone();
                Ok(value)
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