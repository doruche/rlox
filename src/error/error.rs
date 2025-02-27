#[derive(Debug)]
pub enum ErrorType {
    LexerError,
    ParserError,
    SemanticError,
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
            LexerError => write!(f, "Lexical Error at line {}: {}", self.line, self.message),
            ParserError => write!(f, "Parsing Error at line {}: {}", self.line, self.message),
            SemanticError => write!(f, "Semantic Error at line {}: {}", self.line, self.message),
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

    pub fn number_operator_error(line: usize) -> Self {
        Error::new(RuntimeError, 
            "Operand must be a number".to_string(), line)
    }

    pub fn variavle_undefined_error(name: &str, refed_line: usize) -> Self {
        Error::new(SemanticError,
            format!("Undefined variable '{}'.", name), refed_line)
    }


    pub fn invalid_assign_error(refed_line: usize) -> Self {
        Error::new(ParserError, 
            format!("Invalid assignment target."), refed_line)
    }

    pub fn invalid_call_error(called_line: usize) -> Self {
        Error::new(RuntimeError,
            format!("Can only call functions and classes."), called_line)
    }

    pub fn flow_stmt_error(called_line: usize) -> Self {
        Error::new(SemanticError, 
            format!("Continue and break can only be used in loops."), called_line)
    }

    pub fn invalid_return_error(called_line: usize) -> Self {
        Error::new(SemanticError,
            format!("Can't return outside of functions."), called_line)
    }
}

pub fn report(error: Error) {
    eprintln!("{}", error);
}

/*pub fn report_and_suspend(error: Error) -> ! {
    eprintln!("{}", error);
    panic!();
}*/