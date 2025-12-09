use crate::db::Database;
use crate::errors::{ResultResp, ServerError};
use crate::templates;
use astra::Request;

pub fn handle(req: Request, db: &Database) -> ResultResp {
    let method = req.method().as_str();
    let path = req.uri().path();

    match (method, path) {
        ("GET", "/") => templates::homepage(),
        ("GET", "/about") => templates::html("<h1>About</h1>"),
        ("GET", "/hello") => templates::html("<h1>Hello!</h1>"),

        // SQLite test route
        ("GET", "/count") => {
            let count = db.with_conn(|conn| {
                let mut stmt = conn
                    .prepare("SELECT 42")
                    .map_err(|e| ServerError::DbError(format!("Prepare failed: {e}")))?;

                let mut rows = stmt
                    .query([])
                    .map_err(|e| ServerError::DbError(format!("Query failed: {e}")))?;

                let row = rows
                    .next()
                    .map_err(|e| ServerError::DbError(format!("Rows.next failed: {e}")))?
                    .ok_or_else(|| ServerError::DbError("No rows".into()))?;

                let val: i64 = row
                    .get(0)
                    .map_err(|e| ServerError::DbError(format!("Column read failed: {e}")))?;

                Ok(val)
            })?;

            templates::html(&format!("<h1>DB says: {count}</h1>"))
        }

        _ => Err(ServerError::NotFound),
    }
}
