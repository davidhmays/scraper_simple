use crate::db::connection::Database;
use crate::errors::ServerError;
use crate::responses::html_response;
use crate::responses::ResultResp;
use crate::scraper::{RealtorScraper, ScraperError};
use crate::templates;
use astra::{Body, Request, ResponseBuilder};
use maud::html;

pub fn handle(req: Request, db: &Database) -> ResultResp {
    let method = req.method().as_str();
    let path = req.uri().path();

    match (method, path) {
        ("GET", path) if path.starts_with("/static") => serve_static(path),
        ("GET", "/") => html_response(templates::pages::home_page()),
        ("GET", "/admin") => html_response(templates::pages::admin_page()),
        ("GET", "/scrape-test") => {
            let scraper = RealtorScraper::new().map_err(|e| {
                eprintln!("Scraper init error: {e}");
                ServerError::InternalError
            })?;

            let result = scraper
                .fetch_all_properties_paginated(
                    "https://www.realtor.com/realestateandhomes-search/Utah",
                )
                .map_err(|e| {
                    eprintln!("Scrape failed: {e:?}");
                    ServerError::InternalError
                })?;

            let total_properties = result.properties.len();
            let pages_fetched = result.pages_fetched;

            let first_pretty = result
                .properties
                .get(0)
                .and_then(|p| serde_json::to_string_pretty(p).ok())
                .unwrap_or_else(|| "No properties".into());

            let body = maud::html! {
                h1 { "Scrape OK" }

                ul {
                    li { "Pages fetched: " (pages_fetched) }
                    li { "Total properties: " (total_properties) }
                }

                h2 { "First property (debug)" }
                pre { (first_pretty) }
            };

            html_response(body)
        }

        // ("GET", "/about") => templates::html("<h1>About</h1>"),
        // ("GET", "/hello") => templates::html("<h1>Hello!</h1>"),

        // SQLite test route
        // ("GET", "/count") => {
        //     let count = db.with_conn(|conn| {
        //         // 1. Prepare the SQL
        //         let mut stmt = conn
        //             .prepare("SELECT COUNT(*) FROM items")
        //             .map_err(|e| ServerError::DbError(format!("Prepare failed: {e}")))?;

        //         // 2. Run the query (empty params)
        //         let mut rows = stmt
        //             .query([])
        //             .map_err(|e| ServerError::DbError(format!("Query failed: {e}")))?;

        //         // 3. Fetch first row
        //         let row = rows
        //             .next()
        //             .map_err(|e| ServerError::DbError(format!("Rows.next failed: {e}")))?
        //             .ok_or_else(|| ServerError::DbError("No rows".into()))?;

        //         // 4. Extract COUNT(*) as i64
        //         let val: i64 = row
        //             .get(0)
        //             .map_err(|e| ServerError::DbError(format!("Column read failed: {e}")))?;

        //         Ok(val)
        //     })?;

        //     templates::html(&format!("<h1>DB says: {count}</h1>"))
        // }
        // ("GET", "/add") => {
        //     let params = parse_query(&req);
        //     let name = params.get("name").map(String::as_str).unwrap_or("unnamed");

        //     db.with_conn(|conn| {
        //         conn.execute("INSERT INTO items (name) VALUES (?)", [name])
        //             .map_err(|e| ServerError::DbError(format!("Insert failed: {e}")))?;

        //         Ok(())
        //     })?;

        //     templates::html(&format!("<h1>Added {name}</h1>"))
        // }
        _ => Err(ServerError::NotFound),
    }
}

pub fn serve_static(path: &str) -> ResultResp {
    // Strip the leading "/" so paths like "/static/main.css"
    // become "static/main.css"
    let fs_path = &path[1..];

    // Very important: prevent paths like "/static/../../etc/passwd"
    if fs_path.contains("..") {
        return Err(crate::errors::ServerError::BadRequest(
            "Invalid path".into(),
        ));
    }

    // Read the file
    let bytes = std::fs::read(fs_path).map_err(|_| crate::errors::ServerError::NotFound)?;

    // Guess MIME from extension
    let mime = mime_for(fs_path);

    let resp = ResponseBuilder::new()
        .status(200)
        .header("Content-Type", mime)
        .body(Body::from(bytes))
        .unwrap();

    Ok(resp)
}

fn mime_for(path: &str) -> &'static str {
    if path.ends_with(".css") {
        "text/css"
    } else if path.ends_with(".js") {
        "application/javascript"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        "image/jpeg"
    } else if path.ends_with(".gif") {
        "image/gif"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".html") {
        "text/html"
    } else if path.ends_with(".txt") {
        "text/plain"
    } else if path.ends_with(".ttf") {
        "font/ttf"
    } else {
        "application/octet-stream"
    }
}

fn parse_query(req: &astra::Request) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();

    if let Some(q) = req.uri().query() {
        for pair in q.split('&') {
            let mut parts = pair.splitn(2, '=');
            if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
                map.insert(k.to_string(), v.to_string());
            }
        }
    }

    map
}

fn map_scraper_error(err: ScraperError) -> ServerError {
    match err {
        ScraperError::Blocked(msg) => ServerError::BadRequest(format!("Scraper blocked: {msg}")),
        ScraperError::Network(msg) => ServerError::BadRequest(format!("Network error: {msg}")),
        ScraperError::MissingNextData => ServerError::InternalError,
        ScraperError::UnexpectedShape(msg) => ServerError::InternalError,
        _ => ServerError::InternalError,
    }
}
