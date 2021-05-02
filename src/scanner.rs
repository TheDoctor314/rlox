use crate::tokens::Token;
struct Scanner {
    src: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    fn new(src: String) -> Self {
        Self {
            src,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    fn scan_tokens(&mut self) -> Vec<Token> {
        todo!()
    }

    fn scan_token(&mut self) {
        todo!()
    }

    fn advance(&mut self) {
        todo!()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.src.len()
    }
}
