// src/auth/sessions.rs
use crate::errors::ServerError;
use base64::Engine;
use rand::{rngs::OsRng, RngCore};
use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};

pub fn create_session(conn: &Connection, user_id: i64, now: i64) -> Result<String, ServerError> {
    let mut raw = [0u8; 32];
    OsRng.fill_bytes(&mut raw);

    let raw_token = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(raw);

    let hash = Sha256::digest(raw_token.as_bytes());
    let expires_at = now + 60 * 60 * 24 * 7; // 7 days

    conn.execute(
        r#"
        insert into sessions (user_id, token_hash, created_at, expires_at)
        values (?, ?, ?, ?)
        "#,
        params![user_id, hash.as_slice(), now, expires_at],
    )
    .map_err(|e| ServerError::DbError(format!("create session failed: {e}")))?;

    Ok(raw_token)
}

pub fn load_user_from_session(
    conn: &Connection,
    raw_token: &str,
    now: i64,
) -> Result<Option<(i64, String)>, ServerError> {
    let hash = Sha256::digest(raw_token.as_bytes());

    conn.query_row(
        r#"
        select u.id, u.email
        from sessions s
        join users u on u.id = s.user_id
        where s.token_hash = ?
          and s.expires_at > ?
          and s.revoked_at is null
        "#,
        params![hash.as_slice(), now],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
    .optional()
    .map_err(|e| ServerError::DbError(format!("session lookup failed: {e}")))
}
