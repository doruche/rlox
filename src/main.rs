#![allow(unused)]
mod lexer;
mod error;

use std::{env, fs, io::{self, BufRead, Write}, process};
use lexer::Lexer;

fn main() {
    let mut args = env::args();
    args.next();
    match args.next() {
        Some(file_name) => {
            if let Some(_) = args.next() {
                eprintln!("Usage: rlox [script]");
                process::exit(1);
            }
            run_file(file_name);
        },
        None => run_prompt(),
    };
}

fn run_file(file_name: String) {
    let src = fs::read_to_string(file_name);
    match src {
        Ok(src) => run(src),
        Err(e) => {
            eprintln!("{e}");
        },
    };
}

fn run_prompt() {
    let mut stdin = io::stdin().lock();
    let mut buf_reader = io::BufReader::new(stdin);
    loop {
        print!("> ");
        io::stdout().flush();
        let mut line = String::new();
        match buf_reader.read_line(&mut line) {
            Ok(0) => process::exit(0),
            Ok(_) => run(line),
            Err(e) => {
                eprintln!("{e}");
                process::exit(1);
            }
        };
    }   
}

fn run(src: String) {
    let mut lexer = Lexer::new(src);
    loop {
        match lexer.next_token() {
            Some(lexer::Token::Eof) => break,
            Some(token) => println!("{token:?}"),
            None => (),
        }
    }
}
