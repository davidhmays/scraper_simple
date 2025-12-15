// src/errors.rs
use std::error::Error;
use std::fmt;

/// Domain-level errors (DB, config, validation, etc.)
#[derive(Debug)]
pub enum AppError {
    DbError(String),
    ConfigError(String),
    ValidationError(String),
    IoError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::DbError(msg) => write!(f, "Database error: {msg}"),
            AppError::ConfigError(msg) => write!(f, "Configuration error: {msg}"),
            AppError::ValidationError(msg) => write!(f, "Validation error: {msg}"),
            AppError::IoError(msg) => write!(f, "I/O error: {msg}"),
        }
    }
}
impl Error for AppError {}

// TODO: Does this belong in responses? Import chain is weird. Here -> responses.errors.rs
/// HTTP-level errors (routing, request issues, DB issues surfaced to client)
#[derive(Debug)]
pub enum ServerError {
    NotFound,
    BadRequest(String),
    DbError(String),
    InternalError,
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerError::NotFound => write!(f, "Not Found"),
            ServerError::BadRequest(msg) => write!(f, "Bad Request: {msg}"),
            ServerError::DbError(msg) => write!(f, "Database Error: {msg}"),
            ServerError::InternalError => write!(f, "Internal Server Error"),
        }
    }
}

impl Error for ServerError {}

// Maybe move scraper error into a scraper folder with scaper code.
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
