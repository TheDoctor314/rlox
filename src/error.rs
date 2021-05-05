#[derive(Debug)]
pub(crate) enum RloxError {
    // Returned if scanner encounters an error
    Lexical(usize, String, String),
    Parse(usize, String, String),
}

pub(crate) type Result<T> = std::result::Result<T, RloxError>;

impl std::fmt::Display for RloxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RloxError::Lexical(ref line, ref msg, ref what) => {
                write!(f, "Lexical Error [line {}] {}: {:?}", line, msg, what)
            }
            RloxError::Parse(ref line, ref msg, ref near) => {
                write!(f, "Parse Error [line {}] {}: {:?}", line, msg, near)
            }
        }
    }
}
