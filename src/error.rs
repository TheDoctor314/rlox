#[derive(Debug)]
pub(crate) enum RloxError {
    // Returned if scanner encounters an error
    Lexical(usize, String, String),
}
