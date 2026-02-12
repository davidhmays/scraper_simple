use crate::errors::ServerError;
use rusqlite::{params, Connection};
use time::OffsetDateTime;

/// Counts downloads for the user in the current calendar month (UTC).
pub fn count_downloads_this_month(
    conn: &Connection,
    user_id: i64,
    now: i64,
) -> Result<i64, ServerError> {
    // Determine start of the current month based on 'now'
    let dt = OffsetDateTime::from_unix_timestamp(now).unwrap_or_else(|_| OffsetDateTime::now_utc());

    // Replace day with 1 and time with midnight to get start of month
    let start_of_month = dt
        .replace_day(1)
        .unwrap_or(dt) // Day 1 is valid for every month, so this is just type safety
        .replace_time(time::Time::MIDNIGHT)
        .unix_timestamp();

    let count: i64 = conn
        .query_row(
            "select count(*) from download_events where user_id = ? and created_at >= ?",
            params![user_id, start_of_month],
            |r| r.get(0),
        )
        .map_err(|e| ServerError::DbError(format!("count downloads failed: {e}")))?;

    Ok(count)
}

/// Records a download event.
pub fn record_download(
    conn: &Connection,
    user_id: i64,
    state: &str,
    now: i64,
) -> Result<(), ServerError> {
    conn.execute(
        "insert into download_events (user_id, state, format, created_at) values (?, ?, 'xlsx', ?)",
        params![user_id, state, now],
    )
    .map_err(|e| ServerError::DbError(format!("record download failed: {e}")))?;
    Ok(())
}

/// Resets (deletes) usage for a user for the current month.
pub fn reset_user_downloads(conn: &Connection, user_id: i64, now: i64) -> Result<(), ServerError> {
    let dt = OffsetDateTime::from_unix_timestamp(now).unwrap_or_else(|_| OffsetDateTime::now_utc());

    let start_of_month = dt
        .replace_day(1)
        .unwrap_or(dt)
        .replace_time(time::Time::MIDNIGHT)
        .unix_timestamp();

    conn.execute(
        "delete from download_events where user_id = ? and created_at >= ?",
        params![user_id, start_of_month],
    )
    .map_err(|e| ServerError::DbError(format!("reset downloads failed: {e}")))?;

    Ok(())
}
