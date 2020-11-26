use std::error;
use std::fmt;

pub type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Clone, PartialEq)]
pub enum ZtlnError {
    Default(String),
    TopicDoesNotExist(String),
    TopicAlreadyExists(String),
    PathAlreadyExists(String, String),
    PathDoesNotExist(String, String),
    ParserError(String, Option<String>),
    LocationError(String),
}

impl fmt::Display for ZtlnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ZtlnError::TopicDoesNotExist(topic)
                                => write!(f, "→ Topic '{}' does not exist", topic),
            ZtlnError::TopicAlreadyExists(topic)
                                => write!(f, "→ Topic '{}' does already exist", topic),
            ZtlnError::PathAlreadyExists(topic, path)
                                => write!(f, "→ Path '{}/{}' does already exist", topic, path),
            ZtlnError::PathDoesNotExist(topic, path)
                                => write!(f, "→ Path {}/{} does not exist", topic, path),
            ZtlnError::ParserError(field, some_msg) 
                                => write!(f, "→ Parser error while reading '{}' field. {}", field, some_msg.as_deref().unwrap_or("")),
            ZtlnError::LocationError(location)
                                => write!(f, "→ Location '{}' does not exist.", location),
            ZtlnError::Default(message) 
                                => write!(f, "→ {}", message),
                                
        }
    }
}

impl std::error::Error for ZtlnError {}
