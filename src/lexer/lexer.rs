use core::num;

use super::{Token, TokenType};
use crate::{common::Value, error::{self, Error, ErrorType}};

pub struct Lexer {
    src: Vec<char>,
    line: usize,
    cur_pos: usize,

    cache: Option<Token>
}

impl Lexer {
    pub fn new(src: String) -> Self {
        Lexer {
            src: src.chars().collect(),
            line: 1,
            cur_pos: 0,
            cache: None,
        }
    }

    pub fn peek_token(&mut self) -> Option<&Token> {
        if self.cache.is_none() {
            self.cache = self.next_token();
        }
        self.cache.as_ref()
    }

    pub fn next_token(&mut self) -> Option<Token> {
        if self.cache.is_some() {
            return self.cache.take();
        }

        use TokenType::*;
        let info = if let Some(ch) = self.advance() {
            match ch {
                '(' => LParen,
                ')' => Rparen,
                '{' => LBrace,
                '}' => RBrace,
                '+' => Plus,
                '-' => Minus,
                ':' => Colon,
                ';' => SemiColon,
                ',' => Comma,
                '.' => Dot,
                '*' => Star,
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
                } else {
                    Slash
                },
                '"' => return Some(Token::new(String, 
                    Some(Value::String(match self.string() {
                        Ok(string) => string,
                        Err(e) => error::report_and_suspend(e),
                    })), self.line)),
                '0'..='9' => return Some(Token::new(Number, Some(Value::Number(self.number())), self.line)),
                _ if ch.is_ascii_alphabetic() || ch == '_' => return Some(self.identifier()),
                _ => {
                    error::report(Error::new(
                        ErrorType::LexerError,
                        format!("Unexpected charatcer {}", ch),
                        self.line
                    ));
                    return None;
                },
            }
        } else {
            TokenType::Eof
        };

        Some(Token::new(info, None, self.line))
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
            "nil" => TokenType::Nil,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "true" => TokenType::True,
            "var" => TokenType::Var,
            "while" => TokenType::While,
            _ => return Token::new(TokenType::Identifier, Some(Value::String(id)), self.line),
        };
        Token::new(kind, None, self.line)
    }
}