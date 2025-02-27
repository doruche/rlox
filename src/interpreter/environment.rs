#![allow(unused)]

use std::{cell::{Ref, RefCell, RefMut}, collections::HashMap, rc::Rc};

use crate::{common::Value, error::Error, parser::Expr};

use super::{builtin_func, control_flow::FlowContext, Interpreter};

#[derive(Debug)]
pub struct Environment {
    context: Rc<RefCell<Context>>,
    globals: Rc<RefCell<Context>>,
    flow_context: FlowContext,
}

#[derive(Debug, Clone)]
pub(crate) struct Context {
    values: HashMap<String, Value>,
    enclosing: Option<Rc<RefCell<Context>>>,
    flow_context: FlowContext,
}

impl Context {
    pub fn new(flow_context: FlowContext, enclosing: Option<Rc<RefCell<Self>>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Context {
            values: HashMap::new(),
            enclosing,
            flow_context,
        }))
    }

    pub fn set(&mut self, name: String, value: Value) -> Option<Value>{
        self.values.insert(name, value).map(|v| v.clone())
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        self.values.get(name).map(|v|  v.clone())
    }

    pub fn contains(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }

    pub fn get_closing(&self) -> Option<Rc<RefCell<Context>>> {
        self.enclosing.clone()
    }
}

impl Environment {
    pub fn new() -> Self {
        let globals = Context::new(FlowContext::Function, None);
        Environment::init_builtin(globals.borrow_mut());
        Self {
            context: globals.clone(),
            globals,
            flow_context: FlowContext::Function,
        }
    }

    fn init_builtin(mut globals: RefMut<Context>) {
        globals.set("clock".to_string(), Value::Callable(Rc::new(builtin_func::Clock)));
        globals.set("sleep".to_string(), Value::Callable(Rc::new(builtin_func::Sleep)));
        globals.set("read_line".to_string(), Value::Callable(Rc::new(builtin_func::ReadLine)));
        globals.set("to_number".to_string(), Value::Callable(Rc::new(builtin_func::ToNumber)));
    }

    pub fn current_context(&self) -> Rc<RefCell<Context>> {
        self.context.clone()
    }

    pub fn capture_context(&self) -> Rc<RefCell<Context>> {
        self.current_context().clone()
    }

    pub fn set_context(&mut self, context: &Rc<RefCell<Context>>) {
        self.flow_context = context.borrow().flow_context;
        self.context = context.clone();
    }

    pub fn enter_closure(&mut self, closure: &Rc<RefCell<Context>>) {
        self.set_context(closure);
        self.enter_scope(FlowContext::Function);
    }

    pub fn exit_closure(&mut self, previous: &Rc<RefCell<Context>>) {
        self.exit_scope();
        self.set_context(previous);
    }

    pub fn enter_scope(&mut self, flow_context: FlowContext) {
        self.flow_context = flow_context;
        self.context = Context::new(flow_context, Some(self.context.clone()));
        //println!("\tENTER SCOPE\n\t{:?}", self.current_context());
    }

    pub fn exit_scope(&mut self) {
        let enclosing = self.context.borrow().get_closing().unwrap();
        self.flow_context = enclosing.borrow().flow_context;
        self.context = enclosing;
        //println!("\tEXIT SCOPE\n\t{:?}", self.contexts.pop().unwrap());
    }

    pub fn define(&mut self, name: String, value: Value)  {
        self.context.borrow_mut().set(name, value);
    }

    pub fn lookup(&self, name: &str, refed_line: usize) -> Result<Value, Error> {
        let mut context = self.context.clone();
        loop {
            if context.borrow().contains(name) {
                return Ok(context.borrow().get(name).unwrap())
            }
            let enclosing = context.borrow().get_closing();
            if let Some(enclosing) = enclosing {
                context = enclosing;
            } else {
                return Err(Error::variavle_undefined_error(name, refed_line));
            }
        }
    }

    pub fn update(&mut self, name: &str, refed_line: usize, value: Value) -> Result<Value, Error> {
        let mut context = self.context.clone();
        loop {
            if context.borrow().contains(name) {
                return Ok(context.borrow_mut().set(name.to_string(), value).unwrap());
            }
            let enclosing = context.borrow().get_closing();
            if let Some(enclosing) = enclosing {
                context = enclosing;
            } else {
                return Err(Error::variavle_undefined_error(name, refed_line));
            }
        }
    }
}

impl Environment {

    pub fn current_flow_context(&self) -> FlowContext {
        self.current_context().borrow().flow_context
    }

    pub fn contains(&self, flow_context: FlowContext) -> bool {
        let mut context = self.context.clone();
        loop {
            if context.borrow().flow_context == flow_context {
                return true;
            }
            let enclosing = context.borrow().get_closing();
            if let Some(enclosing) = enclosing {
                context = enclosing;
            } else {
                return false;
            }
        }
    }
}

impl Environment {
    fn ancestor(&self, distance: usize) -> Rc<RefCell<Context>> {
        let mut context = self.context.clone();
        for i in 0..distance {
            let enclosing = context.borrow().get_closing().unwrap().clone();
            context = enclosing;
        }
        context
    }

    pub fn lookup_at(&self, name: &str, distance: Option<usize>, refed_line: usize) -> Result<Value, Error> {
        if let Some(distance) = distance {
            //println!("\t{name}\t{distance} refef_line: {refed_line}");
            match self.ancestor(distance).borrow().get(name) {
                Some(value) => Ok(value),
                None => unreachable!(),
            }
        } else {
            match self.globals.borrow().get(name) {
                Some(value) => Ok(value),
                None => Err(Error::variavle_undefined_error(name, refed_line))
            }
        }
    }

    pub fn update_at(&mut self, name: &str, distance: Option<usize>, refed_line: usize, value: Value) -> Result<Value, Error> {
        if let Some(distance) = distance {
            match self.ancestor(distance).borrow_mut().set(name.to_string(), value) {
                Some(value) => Ok(value),
                None => Err(Error::variavle_undefined_error(name, refed_line))
            }
        } else {
            match self.globals.borrow_mut().set(name.to_string(), value) {
                Some(value) => Ok(value),
                None => Err(Error::variavle_undefined_error(name, refed_line))
            }
        }
    }
}