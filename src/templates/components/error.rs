use crate::errors::ServerError;
use astra::{Body, Response, ResponseBuilder};
use maud::{html, Markup};

pub fn html_error_response(err: ServerError) -> Response {
    let markup: Markup = html! {
        h1 { "Error" }
        p { (err.to_string()) }
    };

    let rendered = markup.into_string();

    ResponseBuilder::new()
        .status(500)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(Body::from(rendered))
        .unwrap_or_else(|_| Response::new(Body::from("Internal Server Error")))
}
