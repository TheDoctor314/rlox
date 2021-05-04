#[derive(Debug)]
pub(crate) enum RloxError {
    // Returned if scanner encounters an error
    Lexical(usize, String, String),
}

impl std::fmt::Display for RloxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RloxError::Lexical(ref line, ref msg, ref what) => write!(f, "Lexical Error [line {}] {}: {:?}", line, msg, what),
        }
    }
}
