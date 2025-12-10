use crate::errors::{ResultResp, ServerError};
use astra::{Body, Response, ResponseBuilder};

//TODO: Should I avoid the unwrap_or_else?

pub fn homepage() -> ResultResp {
    html("<h1>Home</h1>")
}

pub fn html(content: &str) -> ResultResp {
    ResponseBuilder::new()
        .status(200)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(Body::from(content.to_string()))
        .map_err(|_| ServerError::InternalError)
}

pub fn html_error_response(err: ServerError) -> Response {
    let body = format!("<h1>Error</h1><p>{}</p>", err);

    ResponseBuilder::new()
        .status(500)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(Body::from(body))
        .unwrap_or_else(|_| Response::new(Body::from("Internal Server Error")))
}
