use std::error;
use std::fmt;

pub type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Clone, PartialEq)]
pub enum ZtlnError {
    Default(String),
    FieldDoesNotExist(String),
    FieldAlreadyExists(String),
    PathAlreadyExists(String, String),
    PathDoesNotExist(String, String),
}

impl fmt::Display for ZtlnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ZtlnError::FieldDoesNotExist(field)
                                => write!(f, "→ Field '{}' does not exist", field),
            ZtlnError::FieldAlreadyExists(field)
                                => write!(f, "→ Field '{}' does already exist", field),
            ZtlnError::PathAlreadyExists(field, path)
                                => write!(f, "→ Path '{}/{}' does already exist", field, path),
            ZtlnError::PathDoesNotExist(field, path)
                                => write!(f, "→ Path {}/{} does not exist", field, path),
            ZtlnError::Default(message) 
                                => write!(f, "→ {}", message),
                                
        }
    }
}

impl std::error::Error for ZtlnError {}
