use rusqlite::Connection;
use std::cell::RefCell;
use std::fs;

use crate::errors::ServerError;

// Thread-local connection slot.
thread_local! {
    static DB_CONN: RefCell<Option<Connection>> = RefCell::new(None);
}

#[derive(Clone)]
pub struct Database {
    path: String,
}

impl Database {
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }

    /// Provides a mutable connection to the closure.
    pub fn with_conn<F, T>(&self, f: F) -> Result<T, ServerError>
    where
        F: FnOnce(&mut Connection) -> Result<T, ServerError>,
    {
        let inner_result = DB_CONN
            .try_with(|cell| {
                let mut slot = cell.borrow_mut();
                if slot.is_none() {
                    let conn = Connection::open(&self.path)
                        .map_err(|e| ServerError::DbError(format!("Open DB failed: {e}")))?;
                    *slot = Some(conn);
                }
                let conn = slot.as_mut().unwrap(); // ✅ mutable reference
                f(conn)
            })
            .map_err(|_| ServerError::InternalError)?;
        inner_result
    }

    /// Example: simple init
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

/// Initialize database from a SQL schema file
pub fn init_db(db: &Database, schema_path: &str) -> Result<(), ServerError> {
    let schema_sql = fs::read_to_string(schema_path)
        .map_err(|e| ServerError::DbError(format!("Failed to read schema file: {e}")))?;

    db.with_conn(|conn| {
        conn.execute_batch(&schema_sql)
            .map_err(|e| ServerError::DbError(format!("Failed to apply schema: {e}")))?;
        Ok(())
    })?;

    println!("✅ Database initialized successfully from {}", schema_path);
    Ok(())
}
