// src/errors.rs
use rust_xlsxwriter::XlsxError;
use std::error::Error;
use std::fmt;

// TODO: Does this belong in responses? Import chain is weird. Here -> responses.errors.rs
/// HTTP-level errors (routing, request issues, DB issues surfaced to client)
// TODO: May be duplicate of responses.error for http related errors.
#[derive(Debug)]
pub enum ServerError {
    NotFound,
    BadRequest(String),
    Unauthorized(String),
    DbError(String),
    InternalError,
    XlsxError(String),
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerError::NotFound => write!(f, "Not Found"),
            ServerError::BadRequest(msg) => write!(f, "Bad Request: {msg}"),
            ServerError::Unauthorized(msg) => write!(f, "Unauthorized: {msg}"),
            ServerError::DbError(msg) => write!(f, "Database Error: {msg}"),
            ServerError::InternalError => write!(f, "Internal Server Error"),
            ServerError::XlsxError(msg) => write!(f, "XLSX Error: {msg}"),
        }
    }
}

impl Error for ServerError {}

impl From<rusqlite::Error> for ServerError {
    fn from(err: rusqlite::Error) -> Self {
        ServerError::DbError(err.to_string())
    }
}

impl From<XlsxError> for ServerError {
    fn from(err: XlsxError) -> Self {
        ServerError::XlsxError(err.to_string())
    }
}
