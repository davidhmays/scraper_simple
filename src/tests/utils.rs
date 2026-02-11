use crate::db::connection::init_db; // <-- correct path to init_db
use crate::db::connection::Database; // <-- your Database type
use astra::Request;
use std::env;
use std::path::PathBuf;

/// Initialize a fresh test DB using your production schema
pub fn init_test_db() -> Database {
    let db = Database::new("test_db.sqlite"); // simple &str path

    init_db(&db, "sql/schema.sql")
        .unwrap_or_else(|e| panic!("Database initialization failed: {e}"));

    db
}
