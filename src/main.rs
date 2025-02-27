mod lexer;
mod error;
mod parser;
mod resolver;
mod interpreter;
mod common;
mod repl;

use std::{env, process};
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
            repl::run_file(file_name, &mut interpreter);
        },
        None => repl::run_prompt(&mut interpreter),
    };
}