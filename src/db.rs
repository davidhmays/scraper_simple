// db.rs
use rusqlite::{Connection, Error as SqlError};
use std::cell::RefCell;

use crate::errors::ServerError;

// Thread-local connection slot.
thread_local! {
    static DB_CONN: RefCell<Option<Connection>> = RefCell::new(None);
}

pub struct Database {
    path: String,
}

impl Database {
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }

    /// Open or fetch the per-thread SQLite connection and run `f(conn)`
    pub fn with_conn<F, T>(&self, f: F) -> Result<T, ServerError>
    where
        F: FnOnce(&Connection) -> Result<T, ServerError>,
    {
        // Step 1: run inside TLS
        let inner_result = DB_CONN
            .try_with(|cell| {
                let mut slot = cell.borrow_mut();

                // Initialize on first use in this thread
                if slot.is_none() {
                    let conn = Connection::open(&self.path)
                        .map_err(|e| ServerError::DbError(format!("Open DB failed: {e}")))?;
                    *slot = Some(conn);
                }

                let conn = slot.as_ref().unwrap();

                // Return the user function's Result<T, ServerError>
                f(conn)
            })
            // Step 2: Map TLS access failure (rare)
            .map_err(|_| ServerError::InternalError)?;

        // Step 3: unwrap the inner Result<T,ServerError>
        inner_result
    }

    /// Example: CREATE TABLE IF NOT EXISTS
    pub fn init(&self) -> Result<(), ServerError> {
        self.with_conn(|conn| {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS items (id INTEGER PRIMARY KEY, name TEXT NOT NULL)",
                [],
            )
            .map_err(|e| ServerError::DbError(format!("Init failed: {e}")))?;

            Ok(())
        })
    }
}
