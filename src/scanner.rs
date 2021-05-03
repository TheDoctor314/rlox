use std::{collections::VecDeque, str::Chars};

use crate::{
    error::RloxError,
    tokens::{Literal, Token, TokenType},
};

type Result<T> = std::result::Result<T, RloxError>;
// Scanner is an iterator which consumes a Chars iterator.
// It returns a Result<Token>.
pub(crate) struct Scanner<'a> {
    src: Chars<'a>,
    queue: VecDeque<char>,
    lexeme: String,
    line: usize,
    offset: usize,
    eof: bool,
}

impl<'a> Scanner<'a> {
    pub fn new(src: Chars<'a>) -> Self {
        Self {
            src,
            queue: VecDeque::new(),
            lexeme: String::new(),
            line: 1,
            offset: 0,
            eof: false,
        }
    }

    fn advance(&mut self) -> Option<char> {
        if self.eof {
            return None;
        }

        let ch = match self.queue.len() {
            0 => self.src.next(),
            _ => self.queue.pop_front(),
        };

        let ch = match ch {
            None => {
                self.eof = true;
                Some('\0')
            }
            Some(ch) => Some(ch),
        };

        if let Some(ch) = ch {
            self.lexeme.push(ch);
            self.offset += 1;
            return Some(ch);
        }
        None
    }

    fn peek(&mut self) -> char {
        if self.queue.len() == 0 {
            if let Some(ch) = self.src.next().or(Some('\0')) {
                self.queue.push_back(ch);
            }
        }

        self.queue[0]
    }

    // Conditional advance(). Advances only when true.
    fn match_advance(&mut self, ch: char) -> bool {
        if self.peek() == ch {
            self.advance().unwrap();
            return true;
        }
        false
    }

    fn advance_until(&mut self, charset: &[char]) -> char {
        let mut last = '\0';

        loop {
            match self.peek() {
                ch if charset.contains(&ch) => break last,
                '\0' => break last,
                ch => {
                    last = ch;
                    self.advance();
                }
            }
        }
    }
}

impl<'a> Scanner<'a> {
    fn token(&mut self, token_type: TokenType, literal: Option<Literal>) -> Option<Result<Token>> {
        Some(Ok(Token::new(
            token_type,
            self.lexeme.clone(),
            literal,
            self.line,
            self.offset - self.lexeme.len(),
        )))
    }

    fn match_token(
        &mut self,
        expected: char,
        t: (TokenType, Option<Literal>),
        u: (TokenType, Option<Literal>),
    ) -> Option<Result<Token>> {
        if self.match_advance(expected) {
            self.token(t.0, t.1)
        } else {
            self.token(u.0, u.1)
        }
    }

    fn string(&mut self) -> Option<Result<Token>> {
        loop {
            let last = self.advance_until(&['\n', '"']);
            match self.peek() {
                '\0' => todo!(), // return err, implement later
                // remove trailing slash for multiline strings
                '"' if last == '\\' => {
                    self.lexeme.pop();
                }
                '"' => break,
                '\n' => self.line += 1,
                _ => todo!(), //unexpected
            };

            self.advance();
        }

        self.advance();

        // Remove the first and last char (double quotes)
        let literal = self
            .lexeme
            .clone()
            .chars()
            .skip(1)
            .take(self.lexeme.len() - 2)
            .collect::<String>();

        self.token(TokenType::StringLiteral, Some(Literal::String(literal)))
    }
}
impl<'a> Iterator for Scanner<'a> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        use TokenType::*;

        if self.eof {
            return None;
        }

        self.lexeme.clear();

        loop {
            match self.advance().unwrap() {
                '\0' => {
                    self.eof = true;
                    return self.token(TokenType::Eof, None);
                }

                '(' => return self.token(LParen, None),
                ')' => return self.token(RParen, None),
                '{' => return self.token(LBrace, None),
                '}' => return self.token(RBrace, None),
                ',' => return self.token(Comma, None),
                '.' => return self.token(Dot, None),
                '-' => return self.token(Minus, None),
                '+' => return self.token(Plus, None),
                ';' => return self.token(SemiColon, None),
                '*' => return self.token(Star, None),

                '!' => return self.match_token('=', (BangEqual, None), (Bang, None)),
                '=' => return self.match_token('=', (EqualEqual, None), (Equal, None)),
                '>' => return self.match_token('=', (GreaterEqual, None), (Greater, None)),
                '<' => return self.match_token('=', (LessEqual, None), (Less, None)),

                '/' => match self.peek() {
                    // Advance until the end of line to ignore text in comment
                    '/' => {
                        self.advance_until(&['\n']);
                        self.lexeme.clear();
                    }
                    _ => return self.token(Slash, None),
                },

                // Ignore all whitespace
                c if c.is_whitespace() => {
                    self.lexeme.clear();
                    if let '\n' = c {
                        self.offset = 0;
                        self.line += 1;
                    }
                }

                '"' => return self.string(),

                _ => return Some(Err(RloxError::Lexical(self.line))),
            }
        }
    }
}
