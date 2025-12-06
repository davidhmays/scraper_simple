use astra::Server;
use std::net::SocketAddr;

use crate::router::handle;

mod db;
mod errors;
mod router;
mod spreadsheet;
mod templates;

fn main() {
    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();

    println!("Starting server at http://{addr}");

    // If the port is taken, this will panic.
    // But once the server is running, Ctrl-C shuts it down cleanly.
    let server = Server::bind(&addr).max_workers(8);

    if let Err(e) = server.serve(|req, _info| match handle(req) {
        Ok(resp) => resp,
        Err(err) => templates::html_error_response(err),
    }) {
        eprintln!("Server ended with error: {e}");
    }

    println!("Server shut down cleanly.");
}
