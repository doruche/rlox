#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // single-character
    LParen, Rparen,
    LBrace, RBrace,
    Comma, Dot, Minus, Plus,
    SemiColon, Slash, Star, Colon,

    // one or two character
    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,

    // literals
    Identifier(String),
    String(String),
    Number(f64),

    // keywords
    And, Class, Else, False, Fun, For, If, Nil, Or,
    Print, Return, Super, This, True, Var, While,

    Eof,
}