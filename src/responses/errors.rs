use crate::errors::ServerError;
use astra::{Body, Response, ResponseBuilder};

pub type ResultResp = Result<Response, ServerError>;

/// Convert a ServerError into a proper HTML response
pub fn error_to_response(err: ServerError) -> Response {
    match err {
        ServerError::NotFound => html_error_response(404, "Not Found"),
        ServerError::BadRequest(msg) => html_error_response(400, &msg),
        ServerError::DbError(msg) => html_error_response(500, &msg),
        ServerError::InternalError => html_error_response(500, "Internal Server Error"),
    }
}

/// Build an HTML error page
pub fn html_error_response(status: u16, message: &str) -> Response {
    let html = format!(
        "<!DOCTYPE html>
        <html lang=\"en\">
        <head><meta charset=\"utf-8\"><title>Error {status}</title></head>
        <body>
            <h1>Error {status}</h1>
            <p>{message}</p>
        </body>
        </html>"
    );

    ResponseBuilder::new()
        .status(status)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(Body::from(html))
        .unwrap()
}
