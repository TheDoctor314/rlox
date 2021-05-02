use std::fmt;

use std::fmt::Display;
#[derive(Display, Debug)]
pub(crate) enum TokenType {
    // Single Character
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    SemiColon,
    Slash,
    Star,

    // One or two character
    Bang,
    Equal,
    BangEqual,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Ident,
    StringLiteral,
    Number,

    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

pub(crate) struct Token {
    token_type: TokenType,
    lexeme: String,
    line: u32,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.token_type, self.lexeme)
    }
}

impl Token {
    pub(crate) fn new(token_type: TokenType, lexeme: String, line: u32) -> Self {
        Self {
            token_type,
            lexeme,
            line,
        }
    }
}
