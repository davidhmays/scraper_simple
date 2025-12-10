use crate::responses::ResultResp;
use astra::{Body, Response, ResponseBuilder};
use maud::Markup; // <-- your alias: Result<Response, ServerError>

pub fn html_response(markup: Markup) -> ResultResp {
    let body = markup.into_string();

    let resp = ResponseBuilder::new()
        .status(200)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(Body::from(body))
        .unwrap();

    Ok(resp)
}
