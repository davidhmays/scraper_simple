use astra::Response;
// errors.rs
use std::fmt;

/// Errors originating from either the server logic
/// (routing, missing resources, etc.) or downstream layers (DB).
#[derive(Debug)]
pub enum ServerError {
    NotFound,
    BadRequest(String),
    DbError(String),
    InternalError,
}

// Type alias commonly used by route handlers.
pub type ResultResp = Result<Response, ServerError>;

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

impl std::error::Error for ServerError {}
