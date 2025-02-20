#![allow(unused)]

#[derive(Debug)]
pub enum ErrorType {
    LexerError,
    ParserError,
    RuntimeError,
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorType,
    message: String,
    line: usize,
}

use ErrorType::*;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            LexerError => write!(f, "Lexer Error at line {}: {}", self.line, self.message),
            ParserError => write!(f, "Parser Error at line {}: {}", self.line, self.message),
            RuntimeError => write!(f, "Runtime Error at line {}: {}", self.line, self.message),
        }
    }
}

impl std::error::Error for Error { }

impl Error {
    pub fn new(kind: ErrorType, message: String, line: usize) -> Error {
        Error {
            kind,
            message,
            line,
        }
    }
}

pub fn report(error: Error) {
    eprintln!("{}", error);
}

pub fn report_and_suspend(error: Error) -> ! {
    eprintln!("{}", error);
    panic!();
}