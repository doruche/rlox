#![allow(unused)]
mod lexer;
mod error;
mod parser;
mod interpreter;
mod common;

use std::{env, fs, io::{self, BufRead, Write}, process};
use lexer::{Lexer, TokenType};
use parser::{Parser, Expr};
use interpreter::Interpreter;


fn main() {
    let mut interpreter = Interpreter::new();

    let mut args = env::args();
    args.next();
    match args.next() {
        Some(file_name) => {
            if let Some(_) = args.next() {
                eprintln!("Usage: rlox [script]");
                process::exit(1);
            }
            run_file(file_name, &mut interpreter);
        },
        None => run_prompt(&mut interpreter),
    };
}

fn run_file(file_name: String, i: &mut Interpreter) {
    let src = fs::read_to_string(file_name);
    match src {
        Ok(src) => run(src, i),
        Err(e) => {
            eprintln!("{e}");
        },
    };
}

fn run_prompt(i: &mut Interpreter) {
    let mut stdin = io::stdin().lock();
    let mut buf_reader = io::BufReader::new(stdin);
    loop {
        print!("> ");
        io::stdout().flush();
        let mut line = String::new();
        match buf_reader.read_line(&mut line) {
            Ok(0) => process::exit(0),
            Ok(_) => if matches!(&line[..], "quit\r\n"|"exit\r\n") {
                process::exit(0);
            } else  {
                run(line.trim_end().to_string(), i);
            },
            Err(e) => {
                eprintln!("{e}");
                process::exit(1);
            }
        };
    }   
}

fn run(src: String, interpreter: &mut Interpreter) {
    let lexer = Lexer::new(src);
    let tokens = lexer.lex();
    if tokens.is_none() {
        return;
    }
    let parser = Parser::new(tokens.unwrap());
    let ast = parser.parse();
    if ast.is_err() {
        return;
    }

    let res = interpreter.eval(&ast.ok().unwrap());
    match res {
        Ok(v) => println!("{v}"),
        Err(e) => error::report(e),
    }
}
