#![allow(unused)]

use std::{f32::consts::E, fs, io::{self, BufRead, Write}, process};
use crate::{common::Value, lexer::{self, Lexer}, resolver::Resolver};
use crate::parser::Parser;
use crate::interpreter::Interpreter;
use crate::error::{self, Error};

pub fn run_file(file_name: String, i: &mut Interpreter) {
    let src = fs::read_to_string(file_name);
    match src {
        Ok(src) => {
            match exec(&src, i) {
                Ok(_) => (),
                Err(errors) => {
                    for error in errors {
                        error::report(error);
                    }
                }
            }
        },
        Err(e) => {
            eprintln!("{e}");
        },
    };
}

pub fn run_prompt(i: &mut Interpreter) {
    let stdin = io::stdin().lock();
    let mut buf_reader = io::BufReader::new(stdin);
    loop {
        print!(">> ");
        io::stdout().flush();
        let mut line = String::new();
        match buf_reader.read_line(&mut line) {
            Ok(0) => process::exit(0),
            Ok(_) => if matches!(&line[..], "quit\r\n"|"exit\r\n") {
                process::exit(0);
            } else  {
                match exec(&line.trim_end(), i) {
                    Ok(_) => (),
                    Err(_) => match eval(&line.trim_end(), i) {
                        Ok(value) => println!("{value}"),
                        Err(errors) => {
                            for error in errors {
                                error::report(error);
                            }
                        }
                    }
                }
            },
            Err(e) => {
                eprintln!("{e}");
                process::exit(1);
            }
        };
    }   
}

fn eval(src: &str, interpreter: &mut Interpreter) -> Result<Value, Vec<Error>> {
    let tokens = match Lexer::new(src).lex() {
        Ok(t) => t,
        Err(errors) => return Err(errors),
    };

    let parser = Parser::new(tokens);
    let mut expr = match parser.parse_expr() {
        Ok(v) => v,
        Err(error) => {
            return Err(vec![error])
        },
    };

    let mut resolver = Resolver::new(interpreter);
    if let Err(e) = resolver.resolve_expr(&mut expr) {
        return Err(vec![e]);
    }

    match interpreter.eval(&expr) {
        Ok(v) => Ok(v),
        Err(e) => Err(vec![e]),
    }
}

fn exec(src: &str, interpreter: &mut Interpreter) -> Result<(), Vec<Error>> {
    let tokens = match Lexer::new(src).lex() {
        Ok(t) => t,
        Err(errors) => return Err(errors),
    };

    let parser = Parser::new(tokens);
    let mut ast = match parser.parse_prog() {
        Ok(v) => v,
        Err(errors) => {
            return Err(errors);
        }
    };

    let mut resolver = Resolver::new(interpreter);
    if let Err(e) = resolver.resolve_stmt(&mut ast) {
        return Err(vec![e]);
    }

    let res = interpreter.exec(&ast);
    match res {
        Ok(v) => Ok(()),
        Err(e) => Err(vec![e]),
    }
}