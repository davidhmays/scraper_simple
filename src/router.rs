use crate::errors::{ResultResp, ServerError};
use crate::templates;
use astra::Request;

pub fn handle(req: Request) -> ResultResp {
    let method = req.method().as_str();
    let path = req.uri().path();

    match (method, path) {
        ("GET", "/") => templates::homepage(),
        ("GET", "/about") => templates::html("<h1>About</h1>"),
        ("GET", "/hello") => templates::html("<h1>Hello!</h1>"),

        _ => Err(ServerError::NotFound),
    }
}
