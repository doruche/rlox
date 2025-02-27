#![allow(unused)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;

use crate::common::Value;
use crate::error::Error;
use crate::parser::Stmt;
use super::environment::Context;
use super::{FlowContext, FlowSignal};
use super::Interpreter;

pub trait Callable : Debug {
    fn call(&self, arguments: Vec<Value>, interpreter: &mut Interpreter, called_line: usize) -> Result<Value, Error>;

    fn arity(&self) -> u8;

    fn to_string(&self) -> String;
}


#[derive(Debug)]
pub struct Function {
    name: Option<String>,
    defined_line: usize,
    params: Vec<String>,
    body: Vec<Stmt>,
    closure: Rc<RefCell<Context>>,
}

#[derive(Debug)]
pub(crate) struct Class {
    pub(crate) name: String,
    pub(crate) defined_line: usize,
}


impl Callable for Function {
    fn call(&self, arguments: Vec<Value>, interpreter: &mut Interpreter, called_line: usize) -> Result<Value, Error> {
        let previous = interpreter.environment.current_context();
        interpreter.environment.enter_closure(&self.closure);
        for (param, arg) in self.params.iter().zip(arguments) {
            interpreter.environment.define(param.to_string(), arg);
        }

        //println!("{:?}", interpreter.environment.current_context());
        let value = match interpreter.exec(&self.body)? {
            FlowSignal::Return(value) => value,
            _ => Value::Nil,
        };

        interpreter.environment.exit_closure(&previous);

        Ok(value)
    }

    fn arity(&self) -> u8 {
        self.params.len() as u8
    }

    fn to_string(&self) -> String {
        match &self.name {
            Some(name) => format!("<fn {name}>"),
            None => format!("<closure>"),
        }
    }
}

impl Function {
    pub fn new(name: Option<String>, defined_line: usize, params: Vec<String>, body: Vec<Stmt>
    , closure: Rc<RefCell<Context>>) -> Self {
        Function {
            name,
            defined_line,
            params,
            body,
            closure,
        }
    }
}

impl Callable for Rc<Class> {
    fn call(&self, arguments: Vec<Value>, interpreter: &mut Interpreter, called_line: usize) -> Result<Value, Error> {
        let instance = Value::Instance {
            class: self.clone(),
            fields: Rc::new(RefCell::new(HashMap::new())),
        };
        Ok(instance)
    }

    fn arity(&self) -> u8 {
        0
    }

    fn to_string(&self) -> String {
        self.name.clone()
    }
}

impl Class {
    pub fn new(name: String, defined_line: usize) -> Rc<Self> {
        Rc::new(Self {
            name,
            defined_line,
        })
    }
}