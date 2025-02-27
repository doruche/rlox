#![allow(unused)]

use std::collections::HashMap;

use crate::{error::Error, interpreter::Interpreter, parser::{Expr, Stmt}};

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    in_loop: bool,
}

impl<'a> Resolver<'a> {
    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn declare(&mut self, name: String) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), false);
        }
    }
    
    pub fn define(&mut self, name: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), true);
        }
    }
}

impl<'a> Resolver<'a>{
    pub fn new(interpreter: &'a mut Interpreter) -> Self {
        Self {
            interpreter,
            scopes: vec![],
            in_loop: false,
        }
    }

    pub fn resolve_stmt(&mut self, statements: &mut Vec<Stmt>) -> Result<(), Error> {
        for stmt in statements {
            match stmt {
                Stmt::Block(block) => {
                    self.enter_scope();
                    self.resolve_stmt(block)?;
                    self.exit_scope();
                },
                Stmt::ClassDecl { 
                    name, 
                    defined_line, 
                    methods 
                } => {
                    self.declare(name.to_string());
                    self.define(&name);
                    self.resolve_stmt(methods)?;
                }
                Stmt::VarDecl { 
                    variable, 
                    defined_line, 
                    initializer 
                } => {
                    self.declare(variable.clone());
                    self.resolve_expr(initializer)?;
                    self.define(&variable);
                },
                Stmt::FunDecl { 
                    name, 
                    defined_line, 
                    params, 
                    body 
                } => {
                    let previous = self.in_loop;
                    self.in_loop = false;
                    self.declare(name.clone());
                    self.define(&name);

                    self.enter_scope();
                    for param in params {
                        self.declare(param.clone());
                        self.define(&param);
                    }
                    self.resolve_stmt(body)?;

                    self.exit_scope();
                    self.in_loop = previous;
                },
                Stmt::Expression(expr) => self.resolve_expr(expr.as_mut())?,
                Stmt::If {
                    condition, 
                    then_branch, 
                    else_branch 
                } => {
                    self.enter_scope();
                    self.resolve_expr(condition)?;
                    self.resolve_stmt(then_branch)?;
                    if let Some(else_branch) = else_branch {
                        self.resolve_stmt(else_branch)?;
                    }
                    self.exit_scope();
                },
                Stmt::Print(expr) => self.resolve_expr(expr)?,
                Stmt::Return { value, line } => self.resolve_expr(value)?,
                Stmt::While {
                    condition, 
                    body, 
                    increment 
                } => {
                    self.in_loop = true;
                    self.enter_scope();
                    self.resolve_expr(condition)?;
                    self.resolve_stmt(body)?;
                    if increment.is_some() {
                        self.resolve_expr(increment.as_deref_mut().unwrap())?;
                    }

                    self.exit_scope();
                    self.in_loop = false;
                }
                Stmt::Break(line)|Stmt::Continue(line) => {
                    if !self.in_loop {
                        return Err(Error::flow_stmt_error(*line));
                    }
                },
                Stmt::Empty => (),
            };
        }
        Ok(())
    }

    pub fn resolve_expr(&mut self, expr: &mut Expr) -> Result<(), Error> {
        match expr {
            Expr::Variable {
                name, 
                refed_line, 
                resolve_distance,
            } => {
                // maybe check an subtle initialization.
                //self.resolve_local(expr, name.clone());
                for (distance, scope) in self.scopes.iter().rev().enumerate() {
                    if scope.contains_key(name) {
                        *resolve_distance = Some(distance);
                        break;
                    }
                }
            },
            Expr::Get { 
                object, 
                called_line, 
                name 
            } => {
                self.resolve_expr(object)?;
            },
            Expr::Set { 
                object, 
                called_line, 
                name, 
                value 
            } => {
                self.resolve_expr(object)?;
                self.resolve_expr(value)?;
            }
            Expr::AssignExpr { 
                name, 
                refed_line, 
                value,
                resolve_distance,
            } => {
                self.resolve_expr(value)?;
                for (distance, scope) in self.scopes.iter().rev().enumerate() {
                    if scope.contains_key(name) {
                        *resolve_distance = Some(distance);
                        break;
                    }
                }
            }, 
            Expr::Lambda { 
                params, 
                body, 
                defined_line 
            } => {
                let previous = self.in_loop;
                self.in_loop = false;
                self.enter_scope();
                for param in params {
                    self.declare(param.clone());
                    self.define(&param);
                }
                self.resolve_stmt(body)?;
                self.exit_scope();
                self.in_loop = previous;
            },
            Expr::ConjunctionExpr(exprs) => {
                for expr in exprs {
                    self.resolve_expr(expr)?;
                }
            },
            Expr::TernaryExpr { 
                condition, 
                true_branch, 
                false_branch, 
                line 
            } => {
                self.resolve_expr(condition)?;
                self.resolve_expr(true_branch)?;
                self.resolve_expr(false_branch)?;
            },
            Expr::BinaryExpr { 
                left, 
                op, 
                right, 
                line 
            } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)?;
            },
            Expr::UnaryExpr {
                op, 
                expr, 
                line 
            } => self.resolve_expr(expr)?,
            Expr::Call { 
                callee, 
                called_line, 
                arguments 
            } => {
                self.resolve_expr(callee)?;
                for arg in arguments {
                    self.resolve_expr(arg)?;
                }
            },
            Expr::GroupExpr(expr) => self.resolve_expr(expr)?,
            Expr::Literal(_) => (),
        };
        Ok(())
    }

    pub fn resolve_local(&mut self, expr: &mut Expr, name: String) {
        for (distance, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name) {
                if let Expr::Variable { resolve_distance, .. } = expr {
                    *resolve_distance = Some(distance);
                }
                return;
            }
        }
    }
}

