#![allow(unused)]

use std::collections::HashMap;

use crate::{common::Value, error::Error};

pub struct Environment {
    values: Vec<HashMap<String, Value>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: vec![HashMap::new()],
        }
    }

    pub fn enter_scope(&mut self) {
        self.values.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        self.values.pop();
    }

    pub fn current_values_mut(&mut self) -> &mut HashMap<String, Value> {
        if let Some(scope) = self.values.last_mut() {
            scope
        } else {
            panic!("no active scope");
        }
    }
  
    pub fn current_values(&self) -> &HashMap<String, Value> {
        if let Some(scope) = self.values.last() {
            scope
        } else {
            panic!("no active scope");
        }
    }
    pub fn define(&mut self, name: String, value: Value) -> Option<Value> {
        self.current_values_mut().insert(name, value)
    }

    pub fn lookup(&self, name: &str, refed_line: usize) -> Result<&Value, Error> {
        for scope in self.values.iter().rev() {
            if scope.contains_key(name) {
                return Ok(scope.get(name).unwrap())
            }
        }
        Err(Error::variavle_undefined_error(name, refed_line))
    }

    pub fn lookup_mut(&mut self, name: &str, refed_line: usize) -> Result<&mut Value, Error> {
        for scope in self.values.iter_mut().rev() {
            if scope.contains_key(name) {
                return Ok(scope.get_mut(name).unwrap())
            }
        }
        Err(Error::variavle_undefined_error(name, refed_line))
    }
}