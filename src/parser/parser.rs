use super::*;
use crate::lexer::{Token, TokenType};
use crate::error::{Error, ErrorType};

pub struct Parser {
    pub(crate) tokens: Vec<Token>,
    pub(crate) current_pos: usize,

    pub(crate) has_error: bool,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            current_pos: 0,

            has_error: false,
        }
    }

    pub fn parse_prog(mut self) -> Result<Vec<Stmt>, Vec<Error>> {
        let mut prog = vec![];
        let mut errors = vec![];
        while !self.at_end() {
            match self.declaration() {
                Ok(stmt) => prog.push(stmt),
                Err(e) => errors.push(e),
            }
        }
        if self.has_error {
            return Err(errors);
        }
        Ok(prog)
    }

    pub fn parse_expr(mut self) -> Result<Expr, Error> {
        self.expr()
    }

    pub(crate) fn eat(&mut self, kind: &TokenType, msg: String) -> Result<Token, Error> {
        if self.peek().kind == *kind {
            Ok(self.eat_current())
        } else {
            self.has_error = true;
            Err(Error::new(ErrorType::ParserError, 
                msg, self.previous().line))
        }
    }

    pub(crate) fn eat_current(&mut self) -> Token {
        if !self.at_end() {
            self.current_pos += 1;
        }
        self.previous()
    }

    pub(crate)  fn previous(&mut self) -> Token {
        let cur = self.tokens.get_mut(self.current_pos - 1).unwrap();
        std::mem::replace(cur, Token::def(cur.line))
    }

    pub(crate) fn peek(&self) -> &Token {
        self.tokens.get(self.current_pos).unwrap()
    }

    pub(crate) fn at_end(&self) -> bool {
        self.peek().kind == TokenType::Eof
    }

    pub(crate) fn synchronize(&mut self) {
        use TokenType::*;
        let mut token = self.eat_current();

        while !self.at_end() {
            if token.kind == SemiColon {
                return;
            }

            if matches!(self.peek().kind,
            Class|Fun|Var|For|If|While|Print|Return) {
                return;
            }

            token = self.eat_current();
        }
    }
}