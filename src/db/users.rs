// src/db/users.rs
use crate::errors::ServerError;
use rusqlite::Connection;

pub fn is_user_admin(_conn: &Connection, _user_id: i64) -> Result<bool, ServerError> {
    Ok(false)
}
