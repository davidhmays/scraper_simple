use crate::db::connection::{init_db, Database};
use crate::router::handle;
use astra::Server;
use std::net::SocketAddr;

mod auth;
mod db;
mod domain;
mod errors;
mod geos;
mod mailings;
mod responses;
mod router;
mod scraper;
mod spreadsheets;
mod templates;

#[cfg(test)]
mod tests;

fn main() {
    // 1️⃣ Create the database handle
    let db = Database::new("myapp.sqlite3");

    // 2️⃣ Initialize database from schema.sql
    // Make sure you have a schema.sql file in your project root or adjust the path
    if let Err(e) = init_db(&db, "sql/schema.sql") {
        eprintln!("❌ Database initialization failed: {e}");
        std::process::exit(1);
    }

    // 3️⃣ Start the server
    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
    println!("Starting server at http://{addr}");

    let server = Server::bind(&addr).max_workers(8);

    // 4️⃣ Serve requests, passing db handle into closure
    let result = server.serve(move |req, _info| match handle(req, &db) {
        Ok(resp) => resp,
        Err(err) => templates::html_error_response(err),
    });

    if let Err(e) = result {
        eprintln!("Server ended with error: {e}");
    }

    println!("Server shut down cleanly.");
}
