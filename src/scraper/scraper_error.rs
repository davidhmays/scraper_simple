use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ScraperError {
    Network(String),
    Blocked(String),
    HtmlParse(String),
    MissingNextData,
    JsonParse(String),
    UnexpectedShape(String),
}

impl fmt::Display for ScraperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScraperError::Network(msg) => write!(f, "Network error: {msg}"),
            ScraperError::Blocked(msg) => write!(f, "Blocked by site: {msg}"),
            ScraperError::HtmlParse(msg) => write!(f, "HTML parse error: {msg}"),
            ScraperError::MissingNextData => write!(f, "__NEXT_DATA__ not found"),
            ScraperError::JsonParse(msg) => write!(f, "JSON parse error: {msg}"),
            ScraperError::UnexpectedShape(msg) => write!(f, "Unexpected data shape: {msg}"),
        }
    }
}

impl Error for ScraperError {}
