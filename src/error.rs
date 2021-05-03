use std::fmt::Display;
#[derive(Display, Debug)]
pub(crate) enum RloxError {
    // Returned if scanner encounters an error
    Lexical(usize),
}