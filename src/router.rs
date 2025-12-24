use crate::db::connection::Database;
use crate::errors::ServerError;
use crate::responses::{html_response, ResultResp};
use crate::scraper::RealtorScraper;
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

        // Spawn scraper background job
        ("GET", "/scrape-test") => {
            let db_clone = db.clone(); // Clone the Database for the thread

            // Spawn background thread
            std::thread::spawn(move || {
                eprintln!("🚀 Background scrape job started");
                RealtorScraper::run_realtor_scrape(&db_clone);
            });

            // Immediately return OK response to browser
            let body = html! {
                h1 { "Scraper triggered in background" }
                p { "Check logs for progress." }
            };
            html_response(body)
        }

        _ => Err(ServerError::NotFound),
    }
}

pub fn serve_static(path: &str) -> ResultResp {
    let fs_path = &path[1..]; // strip leading "/"
    if fs_path.contains("..") {
        return Err(ServerError::BadRequest("Invalid path".into()));
    }

    let bytes = std::fs::read(fs_path).map_err(|_| ServerError::NotFound)?;
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
