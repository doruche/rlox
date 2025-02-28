#![allow(unused)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::error::{Error, ErrorType};
use crate::parser::{Parser, Expr, BinaryOp, UnaryOp, Stmt};
use crate::common::*;

use super::environment::{Context, Environment};
use super::control_flow::*;
use super::callable;

pub struct Interpreter {
    pub environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        let environment = Environment::new();
        
        Self {
            environment,
        }
    }

    pub fn exec(&mut self, statements: &Vec<Stmt>) -> Result<FlowSignal, Error> {
        for stmt in statements {
            match stmt {
                Stmt::ClassDecl { 
                    name, 
                    defined_line, 
                    methods 
                } => {
                    self.environment.define(name.clone(), Value::Nil);
                    let methods = methods.iter()
                        .map(|method| {
                            if let Stmt::FunDecl {
                                name, 
                                defined_line, 
                                params, 
                                body 
                            } = method {
                                callable::Function::new(Some(name.clone()), *defined_line, params.clone(), body.clone(),
                                 Context::new(FlowContext::Block, Some(self.environment.capture_context())))
                            } else {
                                unreachable!();
                            }
                        }).collect();
                    let class = callable::Class::new(name.clone(), *defined_line, methods);
                    self.environment.define(name.clone(), Value::Callable(Rc::new(class)));
                },
                Stmt::FunDecl {
                    name,
                    defined_line,
                    params,
                    body,
                } => {
                    let closure = self.environment.capture_context();
                    self.environment.define(name.clone(), Value::Callable(Rc::new(
                        callable::Function::new(Some(name.clone()), *defined_line, params.clone(), body.clone(), closure)
                    )));
                },
                Stmt::Return {
                    value,
                    line,
                } => {
                    if self.environment.contains(FlowContext::Function) {
                        return Ok(FlowSignal::Return(self.eval(&value)?));
                    } else {
                        return Err(Error::invalid_return_error(*line));
                    }
                },
                Stmt::Break(line) => {
                    if self.environment.contains(FlowContext::Loop) {
                        return Ok(FlowSignal::Break);
                    } else {
                        unreachable!();
                    }
                },
                Stmt::Continue(line) => {
                    if self.environment.contains(FlowContext::Loop) {
                        return Ok(FlowSignal::Continue);
                    } else {
                        unreachable!();
                    }
                },
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
                    body,
                    increment,
                } => {
                    self.environment.enter_scope(FlowContext::Loop);
                    while self.eval(&condition)?.is_truthy() {
                        match self.exec(body)? {
                            FlowSignal::Normal => (),
                            FlowSignal::Break => break,
                            FlowSignal::Continue => {
                                if increment.is_some() {
                                    self.eval(&increment.as_deref().unwrap())?;
                                }
                                continue;
                            },
                            r @FlowSignal::Return {..} => {
                                self.environment.exit_scope();
                                return Ok(r);
                            }
                        }
                    }
                    self.environment.exit_scope();
                },
                Stmt::ForIn { var, array, body, line } => {
                    let array = match self.eval(&array)? {
                        Value::Array(a) => a,
                        _ => return Err(Error::new(ErrorType::RuntimeError, 
                            "For in loops can only be used with array.".to_string(), *line)),
                    };
                    self.environment.enter_scope(FlowContext::Loop);
                    for ele in array.borrow().iter() {
                        self.environment.define(var.clone(), ele.clone());
                        self.exec(body)?;
                    }
                    self.environment.exit_scope();
                },
                Stmt::If {
                    condition, 
                    then_branch, 
                    else_branch 
                } => {
                    self.environment.enter_scope(FlowContext::Block);
                    if self.eval(&condition)?.is_truthy() {
                        match self.exec(then_branch)? {
                            FlowSignal::Normal => (),
                            r @(FlowSignal::Break|FlowSignal::Continue|FlowSignal::Return(..)) => {
                                self.environment.exit_scope();
                                return Ok(r);
                            },                           
                        }
                    } else if else_branch.is_some() {
                        let else_branch = else_branch.as_ref().unwrap();

                        match self.exec(else_branch)? {
                            FlowSignal::Normal => (),
                            r @(FlowSignal::Break|FlowSignal::Continue|FlowSignal::Return(..)) => {
                                self.environment.exit_scope();
                                return Ok(r);
                            },
                        }
                    }
                    self.environment.exit_scope();
                },
                Stmt::Expression(expr) => {
                    let _ = self.eval(expr)?;
                },
                Stmt::Print(expr) => {
                    let v = self.eval(expr)?;
                    println!("{}", v.stringfy());
                },
                Stmt::Block(statements) => {
                    self.environment.enter_scope(FlowContext::Block);
                    match self.exec(statements)? {
                        FlowSignal::Normal => (),
                        r @(FlowSignal::Break|FlowSignal::Continue|FlowSignal::Return(..)) => {
                            self.environment.exit_scope();
                            return Ok(r);
                        },
                    }
                    self.environment.exit_scope();
                },
                Stmt::Empty => (),
            };
        }
        Ok(FlowSignal::Normal)
    }

    pub fn eval(&mut self, expr: &Expr) -> Result<Value, Error> {
        match expr {
            Expr::Literal(value) => Ok(value.clone()),
            Expr::This {
                refed_line, 
                resolve_distance 
            } => self.environment.lookup_at("this", *resolve_distance, *refed_line),
            Expr::Get { 
                object, 
                called_line, 
                name 
            } => {
                let object = self.eval(&object)?;
                if let Value::Instance {
                    class,
                    fields,
                } = &object {
                    if fields.borrow().contains_key(name) {
                        Ok(fields.borrow().get(name).unwrap().clone())
                    } else if let Some(method) = class.find_method(name, object.clone()) {
                        Ok(Value::Callable(method))
                    } else {
                        Err(Error::new(ErrorType::RuntimeError, 
                            format!("Undefined property '{name}'."), *called_line))
                    }
                } else {
                    Err(Error::new(ErrorType::RuntimeError, 
                        "Only instances have properties.".to_string(), *called_line))
                }
            },
            Expr::Set { 
                object, 
                called_line, 
                name, 
                value 
            } => {
                let object = self.eval(&object)?;
                if let Value::Instance { 
                    class, 
                    mut fields 
                } = object {
                    let value = self.eval(&value)?;
                    fields.borrow_mut().insert(name.clone(), value.clone());
                    Ok(value)
                } else {
                    Err(Error::new(ErrorType::RuntimeError, 
                        "Only instances have properties.".to_string(), *called_line))
                }
            },
            Expr::Variable {
                name, 
                refed_line, 
                resolve_distance
            } => self.environment.lookup_at(name, *resolve_distance, *refed_line),
            Expr::Call {
                callee, 
                called_line, 
                arguments 
            } => {
                let callee = self.eval(&callee)?;
                match callee {
                    Value::Callable(callee) => {
                        if arguments.len() as u8 != callee.arity() {
                            return Err(Error::new(ErrorType::RuntimeError, 
                                format!("Expected {} arguments but got {}.", callee.arity(), arguments.len()), *called_line))
                        }
                        let arguments = arguments.iter()
                        .map(|arg| self.eval(arg))
                        .collect::<Result<Vec<Value>, Error>>()?;
                        callee.call(arguments, self, *called_line)
                    },
                    _ => Err(Error::invalid_call_error(*called_line)),
                }
            },
            Expr::Array { 
                refed_line, 
                elements 
            } => {
                let mut array = vec![];
                for e in elements {
                    array.push(self.eval(e)?);
                }
                Ok(Value::Array(Rc::new(RefCell::new(array))))
            },
            Expr::Index {
                array,
                refed_line,
                index,
            } => {
                let array = match self.eval(array)? {
                    Value::Array(a) => a,
                    _ => return Err(Error::new(ErrorType::SemanticError, 
                        "Can only use index expression with array".to_string(), *refed_line)),
                };
                let index = match self.eval(index)? {
                    Value::Number(n) => {
                        if n.ceil() < 0.0 {
                            return Err(Error::new(ErrorType::RuntimeError, 
                                "Indexes must be positive.".to_string(), *refed_line));
                        } else if n.floor() as usize >= array.borrow().len() {
                            return Err(Error::new(ErrorType::RuntimeError, 
                                "Indexes must be in range of array's length.".to_string(), *refed_line));
                        }
                        n
                    },
                    _ => return Err(Error::new(ErrorType::RuntimeError, 
                        "Indexes must be integers.".to_string(), *refed_line)),
                };
                let v = array.borrow().get(index as usize).unwrap().clone();
                Ok(v)
            },
            Expr::IndexSet { array, refed_line, index, value } => {
                let mut array = match self.eval(array)? {
                    Value::Array(a) => a,
                    _ => return Err(Error::new(ErrorType::SemanticError, 
                        "Can only use index expression with array".to_string(), *refed_line)),
                };
                let index = match self.eval(index)? {
                    Value::Number(n) => {
                        if n.ceil() < 0.0 {
                            return Err(Error::new(ErrorType::RuntimeError, 
                                "Indexes must be positive.".to_string(), *refed_line));
                        } else if n.floor() as usize >= array.borrow().len() {
                            return Err(Error::new(ErrorType::RuntimeError, 
                                "Indexes must be in range of array's length.".to_string(), *refed_line));
                        }
                        n
                    },
                    _ => return Err(Error::new(ErrorType::RuntimeError, 
                        "Indexes must be integers.".to_string(), *refed_line)),
                };
                let value = self.eval(&value)?;
                array.borrow_mut()[index as usize] = value.clone();
                Ok(value)
            },
            Expr::Lambda { 
                params, 
                body, 
                defined_line 
            } => {
                Ok(Value::Callable(Rc::new(callable::Function::new(
                    None, *defined_line, params.clone(), body.clone(), self.environment.capture_context()))))
            },
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
            Expr::GroupExpr(expr) => self.eval(&expr),
            Expr::AssignExpr {
                name, 
                refed_line, 
                value ,
                resolve_distance,
            } => {
                let value = self.eval(&value)?;
                self.environment.update_at(name, *resolve_distance, *refed_line, value)
            },
            Expr::ConjunctionExpr(exprs) => {
                let mut res = Value::Nil;
                for cexpr in exprs {
                    res = self.eval(cexpr)?;
                }
                Ok(res)
            }
        }
    }
}