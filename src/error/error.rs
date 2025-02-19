#![allow(unused)]

#[derive(Debug)]
pub enum Error {
    LexerError(String),
    ParserError(String),
    RuntimeError(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for Error { }