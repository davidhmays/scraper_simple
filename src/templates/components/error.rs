use crate::errors::ServerError;
use astra::{Body, Response, ResponseBuilder};

/// Convert a ServerError into a proper HTML response page
pub fn html_error_response(err: ServerError) -> Response {
    match err {
        ServerError::NotFound => render_error(404, "Not Found"),

        ServerError::BadRequest(msg) => render_error(400, &msg),

        ServerError::Unauthorized(msg) => render_error(401, &msg),

        ServerError::DbError(msg) => render_error(500, &format!("Database Error: {msg}")),

        ServerError::InternalError => render_error(500, "Internal Server Error"),

        ServerError::XlsxError(msg) => render_error(500, &format!("Spreadsheet Error: {msg}")),
    }
}

/// Build a basic HTML error page
fn render_error(status: u16, message: &str) -> Response {
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Error {status}</title>
  <style>
    body {{
      font-family: system-ui, sans-serif;
      max-width: 720px;
      margin: 4rem auto;
      padding: 1rem;
    }}
    h1 {{
      font-size: 2rem;
      margin-bottom: 1rem;
    }}
    p {{
      font-size: 1.1rem;
      color: #444;
    }}
    code {{
      background: #f4f4f4;
      padding: 0.2rem 0.4rem;
      border-radius: 6px;
    }}
  </style>
</head>
<body>
  <h1>Error {status}</h1>
  <p>{message}</p>
  <p><a href="/">← Back to home</a></p>
</body>
</html>"#
    );

    ResponseBuilder::new()
        .status(status)
        .header("Content-Type", "text/html; charset=utf-8")
        .body(Body::from(html))
        .unwrap()
}
