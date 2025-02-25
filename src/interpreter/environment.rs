#![allow(unused)]

use std::collections::HashMap;

use crate::{common::Value, error::Error};

use super::control_flow::FlowContext;

#[derive(Debug)]
pub struct Environment {
    contexts: Vec<Context>,
}

#[derive(Debug, Clone)]
pub(crate) struct Context {
    values: HashMap<String, Value>,
    flow_context: FlowContext,
}

impl Context {
    pub fn new(flow_context: FlowContext) -> Self {
        Context {
            values: HashMap::new(),
            flow_context,
        }
    }

    pub fn define(&mut self, name: String, value: Value) -> Option<Value> {
        self.values.insert(name, value)
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.values.get(name)
    }
 
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Value> {
        self.values.get_mut(name)
    }
 
    pub fn update(&mut self, name: String, value: Value) -> Result<(), String> {
        if self.values.contains_key(&name) {
            self.values.insert(name, value);
            Ok(())
        } else {
            Err(format!("Variable '{}' not defined", name))
        }
    }

    pub fn contains(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }

    // 别名方法
    pub fn set(&mut self, name: String, value: Value) -> Option<Value> {
        self.define(name, value)
    }
}

impl Environment {
    pub fn new() -> Self {
        Self {
            contexts: vec![Context::new(FlowContext::Function)],
        }
    }

    pub fn enter_scope(&mut self, flow_context: FlowContext) {
        self.contexts.push(Context::new(flow_context));
        //println!("\tENTER SCOPE\n\t{:?}", self.current_context());
    }

    pub fn exit_scope(&mut self) {
        self.contexts.pop().unwrap();
        //println!("\tEXIT SCOPE\n\t{:?}", self.contexts.pop().unwrap());
    }

    pub fn current_context_mut(&mut self) -> &mut Context {
        if let Some(scope) = self.contexts.last_mut() {
            scope
        } else {
            panic!("no active scope");
        }
    }
  
    pub fn current_context(&self) -> &Context {
        if let Some(scope) = self.contexts.last() {
            scope
        } else {
            panic!("no active scope");
        }
    }


    pub fn define(&mut self, name: String, value: Value) -> Option<Value> {
        self.current_context_mut().define(name, value)
    }

    pub fn lookup(&self, name: &str, refed_line: usize) -> Result<&Value, Error> {
        for scope in self.contexts.iter().rev() {
            if scope.contains(name) {
                return Ok(scope.get(name).unwrap())
            }
        }
        Err(Error::variavle_undefined_error(name, refed_line))
    }

    pub fn lookup_mut(&mut self, name: &str, refed_line: usize) -> Result<&mut Value, Error> {
        for scope in self.contexts.iter_mut().rev() {
            if scope.contains(name) {
                return Ok(scope.get_mut(name).unwrap())
            }
        }
        Err(Error::variavle_undefined_error(name, refed_line))
    }
}

impl Environment {

    pub fn current_flow_context(&self) -> FlowContext {
        self.current_context().flow_context
    }

    pub fn seek_loop(&mut self, called_line: usize) -> Result<(), Error> {
        while self.contexts.len() != 0
        && self.current_flow_context() != FlowContext::Loop {
            self.exit_scope();
        }
        if self.current_flow_context() != FlowContext::Loop {
            Err(Error::flow_stmt_error(called_line))
        } else {
            Ok(())
        }
    }

    pub fn contains(&self, flow_context: FlowContext) -> bool {
        for context in self.contexts.iter().rev() {
            if context.flow_context == flow_context {
                return true;
            }
        }
        return false;    
    }
}