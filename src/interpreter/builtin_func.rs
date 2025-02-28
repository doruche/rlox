#![allow(unused)]

use std::io::BufRead;
use std::{io, thread, time};
use crate::common::Value;
use crate::error::{Error, ErrorType};
use super::{callable, Callable};

#[derive(Debug)]
pub struct Clock;
#[derive(Debug)]
pub struct Sleep;
#[derive(Debug)]
pub struct ReadLine;
#[derive(Debug)]
pub struct ToNumber;
#[derive(Debug)]
pub struct Len;

#[derive(Debug)]
pub struct TypeOf;

impl Callable for Clock {
    fn call(&self, arguments: Vec<crate::common::Value>, interpreter: &mut super::Interpreter, called_line: usize) -> Result<Value, Error> {
        let since_the_epoch = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .expect("Time went backwards");
        let ms = since_the_epoch.as_secs() as i64 * 1000i64 + (since_the_epoch.subsec_nanos() as f64 / 1_000_000.0) as i64;
        Ok(Value::Number(ms as f64))
    }

    fn arity(&self) -> u8 {
        0
    }

    fn to_string(&self) -> String {
        "<native clock>".to_string()
    }
}

impl Callable for Sleep {
    fn call(&self, arguments: Vec<Value>, interpreter: &mut super::Interpreter, called_line: usize) -> Result<Value, Error> {
        if let Some(Value::Number(mil)) = arguments.first() {
            let duration = time::Duration::from_millis(*mil as u64);
            thread::sleep(duration);
            Ok(Value::Nil)
        } else {
            Err(Error::new(ErrorType::RuntimeError, 
                "clock() need a number as its argument.".to_string(), called_line))
        }
    }

    fn arity(&self) -> u8 {
        1
    }

    fn to_string(&self) -> String {
        "<native sleep>".to_string()
    }
}

impl Callable for ReadLine {
    fn call(&self, arguments: Vec<Value>, interpreter: &mut super::Interpreter, called_line: usize) -> Result<Value, Error> {
        let mut stdin = io::stdin().lock();
        let mut line = String::new();
        if let Err(e) = stdin.read_line(&mut line) {
            return Err(Error::new(ErrorType::RuntimeError,
                "read_line() io error".to_string(), called_line));
        }
        Ok(Value::String(line.trim_end().to_string()))
    }

    fn arity(&self) -> u8 {
        0
    }

    fn to_string(&self) -> String {
        "<native read_line>".to_string()
    }
}

impl Callable for ToNumber {
    fn call(&self, arguments: Vec<Value>, interpreter: &mut super::Interpreter, called_line: usize) -> Result<Value, Error> {
        Ok(Value::Number(match arguments.first().unwrap() {
            Value::Boolean(b) => if *b {1.} else {0.},
            Value::Callable(c) => 1.,
            Value::Instance {..} => 1.,
            Value::Nil => 0.,
            Value::Number(n) => *n,
            Value::Array(array) => array.borrow().len() as f64,
            Value::String(str) => {
                if let Ok(number) = str.parse() {
                    number
                } else {
                    return Err(Error::new(ErrorType::RuntimeError,
                        "to_number() parse error".to_string(), called_line));
                }
            }
        }))
    }

    fn arity(&self) -> u8 {
        1
    }

    fn to_string(&self) -> String {
        "<native to_number>".to_string()
    }
}

impl Callable for Len {
    fn call(&self, arguments: Vec<Value>, interpreter: &mut super::Interpreter, called_line: usize) -> Result<Value, Error> {
        if let Some(Value::Array(array)) = arguments.first() {
            Ok(Value::Number(array.borrow().len() as f64))
        } else {
            Err(Error::new(ErrorType::RuntimeError, 
                "len() need an array as its argument.".to_string(), called_line))
        }
    }

    fn arity(&self) -> u8 {
        1
    }

    fn to_string(&self) -> String {
        "<native len>".to_string()
    }
}

impl Callable for TypeOf {
    fn call(&self, arguments: Vec<Value>, interpreter: &mut super::Interpreter, called_line: usize) -> Result<Value, Error> {
        if let Some(value) = arguments.first() {
            Ok(Value::String(match value {
                Value::Nil => "nil",
                Value::Number(..) => "number",
                Value::Array(..) => "array",
                Value::Boolean(..) => "boolean",
                Value::Callable(..) => "callable",
                Value::Instance {..} => "instance",
                Value::String(..) => "string",
            }.to_string()))
        } else {
            Err(Error::new(ErrorType::RuntimeError, 
                "typeof() need an argument.".to_string(), called_line))
        }
    }

    fn arity(&self) -> u8 {
        1
    }

    fn to_string(&self) -> String {
        "<native typeof>".to_string()
    }
}