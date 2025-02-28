use super::{Token, TokenType};
use crate::{common::Value, error::{Error, ErrorType}};

pub struct Lexer {
    src: Vec<char>,
    line: usize,
    cur_pos: usize,

    has_error: bool,
}

impl Lexer {
    pub fn new(src: &str) -> Self {
        Lexer {
            src: src.chars().collect(),
            line: 1,
            cur_pos: 0,
            has_error: false,
        }
    }

    pub fn lex(mut self) -> Result<Vec<Token>, Vec<Error>> {
        let mut tokens = vec![];
        let mut errors = vec![];
        loop {
            match self.next_token() {
                Err(e) => errors.push(e),
                Ok(token) if token.kind == TokenType::Eof => {
                    tokens.push(token);
                    break;
                }
                Ok(token) => tokens.push(token),
            };
        }
        if self.has_error {
            Err(errors)
        } else {
            Ok(tokens)
        }
    }

    pub fn next_token(&mut self) -> Result<Token, Error> {

        use TokenType::*;
        let info = if let Some(ch) = self.advance() {
            match ch {
                '(' => LParen,
                ')' => Rparen,
                '{' => LBrace,
                '}' => RBrace,
                '+' => if self.expect('=') { SelfAdd } else { Plus },
                '-' => if self.expect('=') { SelfSub } else { Minus },
                '*' => if self.expect('=') { SelfMul } else { Star },
                ':' => Colon,
                ';' => SemiColon,
                ',' => Comma,
                '.' => Dot,
                '?' => Question,
                '|' => Pipe,
                '[' => LBlock,
                ']' => RBlock,
                ' '|'\r'|'\t' => return self.next_token(),
                '\n' => {
                    self.line += 1;
                    return self.next_token()
                },
                '!' => if self.expect('=') { BangEqual } else { Bang },
                '=' => if self.expect('=') { EqualEqual } else { Equal },
                '>' => if self.expect('=') { GreaterEqual } else { Greater },
                '<' => if self.expect('=') { LessEqual } else { Less },
                '/' => if self.expect('/') {
                    while !self.at_end() {
                        let ch = self.peek().unwrap();
                        if ch == '\n' {
                            break;
                        }
                        self.advance();
                    }
                    return self.next_token();
                } else if self.expect('=') {
                    SelfDiv
                } else {
                    Slash
                },
                '"' => return Ok(Token::new(String, 
                    Some(Value::String(match self.string() {
                        Ok(string) => string,
                        Err(e) => {
                            self.has_error = true;
                            return Err(e);
                        }
                    })), self.line)),
                '0'..='9' => return Ok(Token::new(Number, Some(Value::Number(self.number())), self.line)),
                _ if ch.is_ascii_alphabetic() || ch == '_' => return Ok(self.identifier()),
                _ => {
                    self.has_error = true;
                    return Err(Error::new(
                        ErrorType::LexerError,
                        format!("Unexpected charatcer {}", ch),
                        self.line
                    ));
                },
            }
        } else {
            TokenType::Eof
        };

        Ok(Token::new(info, None, self.line))
    }
    
    fn at_end(&self) -> bool {
        self.cur_pos == self.src.len()
    }

    fn advance(&mut self) -> Option<char> {
        if self.at_end() {
            None
        } else {
            self.cur_pos += 1;
            Some(self.src[self.cur_pos - 1])
        }
    }

    fn peek(&self) -> Option<char> {
        if self.at_end() {
            None
        } else {
            Some(self.src[self.cur_pos])
        }
    }

    fn peek_next(&self) -> Option<char> {
        if self.cur_pos + 1 == self.src.len() {
            None
        } else {
            Some(self.src[self.cur_pos + 1])
        }
    }

    fn expect(&mut self, expected: char) -> bool {
        if self.at_end()
        || self.src[self.cur_pos] != expected {
            false
        } else {
            self.cur_pos += 1;
            if expected == '\n' {
                self.line += 1;
            }
            true
        }
    }

    fn string(&mut self) -> Result<String, Error> {
        let start_pos = self.cur_pos;
        let mut ch = self.advance().unwrap();
        while !self.at_end() && ch != '"' {
            if ch == '\n' {
                self.line += 1;
            }
            ch = self.advance().unwrap();
        }

        if ch == '"' {
            Ok(self.src[start_pos..self.cur_pos - 1].iter().collect())
        } else {
            Err(Error::new(ErrorType::LexerError, 
                "Unterminated string".to_string(), 
                self.line))
        }
    }

    fn number(&mut self) -> f64 {
        let start_pos = self.cur_pos - 1;
        while !self.at_end() {
            let ch = self.peek().unwrap();
            if ch.is_digit(10) {
                self.advance();
            } else {
                break;
            }
        }
        if !self.at_end() && self.peek().unwrap() == '.' {
            match self.peek_next() {
                Some(ch) => {
                    if ch.is_digit(10) {
                        self.advance();
                        while !self.at_end() {
                            let ch = self.peek().unwrap();
                            if ch.is_digit(10) {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                },
                None => (),
            };
        }
        self.src[start_pos..self.cur_pos].iter().collect::<String>().parse().unwrap()
    }



    fn identifier(&mut self) -> Token {
        let start_pot = self.cur_pos - 1;

        while !self.at_end() {
            let ch = self.peek().unwrap();
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let id: String = self.src[start_pot..self.cur_pos].iter().collect();
        let kind =match &id[..] {
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "for" => TokenType::For,
            "fun" => TokenType::Fun,
            "if" => TokenType::If,
            "in" => TokenType::In,
            "nil" => TokenType::Nil,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "true" => TokenType::True,
            "var" => TokenType::Var,
            "while" => TokenType::While,
            "break" => TokenType::Break,
            "continue" => TokenType::Continue,
            _ => return Token::new(TokenType::Identifier, Some(Value::String(id)), self.line),
        };
        Token::new(kind, None, self.line)
    }
}