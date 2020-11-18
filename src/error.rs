use std::error;
use std::fmt;

pub type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Clone)]
pub struct ZtlnError {
    message: String,
}

impl ZtlnError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl fmt::Display for ZtlnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NoteError → {}", self.message)
    }
}

impl std::error::Error for ZtlnError {}
