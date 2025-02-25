use crate::common::Value;

#[derive(Debug)]
pub struct Token {
    pub kind: TokenType,
    pub lexeme: Option<Value>,
    pub line: usize,
}

impl Token {
    pub fn new(kind: TokenType, lexeme: Option<Value>, line: usize) -> Self {
        Token {
            kind,
            lexeme,
            line,
        }
    }

    pub fn def(line: usize) -> Self {
        Self {
            kind: TokenType::Def,
            lexeme: None,
            line,
        }
    }
}

impl Default for Token {
    fn default() -> Self {
        Self { kind: TokenType::Def, lexeme: Default::default(), line: Default::default() }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenType {
    // single-character
    LParen, Rparen,
    LBrace, RBrace,
    Comma, Dot, Minus, Plus,
    SemiColon, Slash, Star, Colon,
    Question, Pipe,

    // one or two character
    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,

    // literals
    Identifier,
    String,
    Number,

    // keywords
    And, Class, Else, False, Fun, For, If, Nil, Or,
    Print, Return, Super, This, True, Var, While, Break,
    Continue,

    Eof,

    // unused
    Def,
}