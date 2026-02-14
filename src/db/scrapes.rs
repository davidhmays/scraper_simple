use crate::errors::ServerError;
use rusqlite::{params, Connection};

#[derive(Debug)]
pub struct ScrapeRun {
    pub id: i64,
    pub state: String,
    pub started_at: i64,
    pub finished_at: Option<i64>,
    pub pages_fetched: Option<i64>,
    pub properties_seen: Option<i64>,
    pub success: Option<bool>,
    pub error_message: Option<String>,
}

pub fn start_scrape_run(conn: &Connection, state_abbr: &str, now: i64) -> Result<i64, ServerError> {
    conn.execute(
        "INSERT INTO scrape_runs (state, started_at, success) VALUES (?, ?, 0)",
        params![state_abbr, now],
    )
    .map_err(|e| ServerError::DbError(e.to_string()))?;
    Ok(conn.last_insert_rowid())
}

pub fn end_scrape_run(
    conn: &Connection,
    run_id: i64,
    now: i64,
    pages: usize,
    props: usize,
    success: bool,
    error: Option<String>,
) -> Result<(), ServerError> {
    conn.execute(
        "UPDATE scrape_runs SET finished_at = ?, pages_fetched = ?, properties_seen = ?, success = ?, error_message = ? WHERE id = ?",
        params![now, pages, props, success, error, run_id],
    ).map_err(|e| ServerError::DbError(e.to_string()))?;
    Ok(())
}

pub fn get_recent_scrapes(conn: &Connection) -> Result<Vec<ScrapeRun>, ServerError> {
    let mut stmt = conn
        .prepare("SELECT id, state, started_at, finished_at, pages_fetched, properties_seen, success, error_message FROM scrape_runs ORDER BY started_at DESC LIMIT 50")
        .map_err(|e| ServerError::DbError(e.to_string()))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ScrapeRun {
                id: row.get(0)?,
                state: row.get(1)?,
                started_at: row.get(2)?,
                finished_at: row.get(3)?,
                pages_fetched: row.get(4)?,
                properties_seen: row.get(5)?,
                success: row.get(6)?,
                error_message: row.get(7)?,
            })
        })
        .map_err(|e| ServerError::DbError(e.to_string()))?;

    let mut runs = Vec::new();
    for r in rows {
        runs.push(r.map_err(|e| ServerError::DbError(e.to_string()))?);
    }
    Ok(runs)
}
