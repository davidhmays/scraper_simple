use astra::Response;
use std::fmt;

#[derive(Debug)]
pub enum ServerError {
    NotFound,
    //Io(std::io::Error), //Adding later.
    InternalError,
}

//Type alias
pub type ResultResp = Result<Response, ServerError>;

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerError::NotFound => write!(f, "Not Found"),
            //ServerError::Io(e) => write!(f, "IO error: {e}"),
            ServerError::InternalError => write!(f, "Internal Server Error"),
        }
    }
}
