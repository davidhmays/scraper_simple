use crate::db::Database;
use crate::router::handle;
use astra::Server;
use std::net::SocketAddr;

mod db;
mod errors;
mod responses;
mod router;
mod scraper;
mod spreadsheet;
mod templates;

fn main() {
    // Create the database handle
    let db = Database::new("myapp.sqlite3");

    // Run initialization
    db.init().expect("DB init failed");

    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
    println!("Starting server at http://{addr}");

    // Build the server
    let server = Server::bind(&addr).max_workers(8);

    // Move db into the closure so each request can access it
    let result = server.serve(move |req, _info| match handle(req, &db) {
        Ok(resp) => resp,
        Err(err) => templates::html_error_response(err),
    });

    if let Err(e) = result {
        eprintln!("Server ended with error: {e}");
    }

    println!("Server shut down cleanly.");
}
