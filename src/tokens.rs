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

#[derive(Debug)]
pub(crate) struct Token {
    token_type: TokenType,
    lexeme: String,
    literal: Option<Literal>,
    line: usize,
    offset: usize,
}

impl Token {
    pub(crate) fn new(
        token_type: TokenType,
        lexeme: String,
        literal: Option<Literal>,
        line: usize,
        offset: usize,
    ) -> Self {
        Self {
            token_type,
            lexeme,
            literal,
            line,
            offset,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} {} {:?}",
            self.token_type, self.lexeme, self.literal
        )
    }
}

impl Default for Token {
    fn default() -> Self {
        Self {
            token_type: TokenType::Eof,
            lexeme: "".to_string(),
            literal: None,
            line: 0,
            offset: 0,
        }
    }
}

#[derive(Debug, Clone, Display)]
pub(crate) enum Literal {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
}
