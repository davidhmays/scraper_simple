// src/db/auth.rs
use rusqlite::{params, Connection, OptionalExtension};

use crate::errors::ServerError;

#[derive(Debug, Clone)]
pub struct MagicLinkRow {
    pub id: i64,
    pub user_id: i64,
    pub created_at: i64,
    pub expires_at: i64,
    pub used_at: Option<i64>,
}

/// Insert a user if they don't exist, then return the user id.
/// Email should already be normalized by caller (trim/lowercase).
pub fn get_or_create_user(conn: &Connection, email: &str, now: i64) -> Result<i64, ServerError> {
    conn.execute(
        "insert or ignore into users (email, created_at) values (?, ?)",
        params![email, now],
    )
    .map_err(|e| ServerError::DbError(format!("insert user failed: {e}")))?;

    let id: i64 = conn
        .query_row(
            "select id from users where email = ?",
            params![email],
            |row| row.get(0),
        )
        .map_err(|e| ServerError::DbError(format!("select user id failed: {e}")))?;

    Ok(id)
}

/// Ensure a user has an entitlement row (one per user) pointing at a plan code.
/// Typically called with plan_code = "free".
pub fn ensure_entitlement(
    conn: &Connection,
    user_id: i64,
    plan_code: &str,
    now: i64,
) -> Result<(), ServerError> {
    conn.execute(
        "insert or ignore into entitlements (user_id, plan_code, granted_at) values (?, ?, ?)",
        params![user_id, plan_code, now],
    )
    .map_err(|e| ServerError::DbError(format!("insert entitlement failed: {e}")))?;
    Ok(())
}

/// Insert a magic link row (token_hash should be SHA-256 bytes).
pub fn insert_magic_link(
    conn: &Connection,
    user_id: i64,
    token_hash: &[u8],
    created_at: i64,
    expires_at: i64,
) -> Result<(), ServerError> {
    conn.execute(
        "insert into magic_links (user_id, token_hash, created_at, expires_at) values (?, ?, ?, ?)",
        params![user_id, token_hash, created_at, expires_at],
    )
    .map_err(|e| ServerError::DbError(format!("insert magic link failed: {e}")))?;
    Ok(())
}

/// Find magic link by token hash.
pub fn find_magic_link_by_hash(
    conn: &Connection,
    token_hash: &[u8],
) -> Result<Option<MagicLinkRow>, ServerError> {
    let row = conn
        .query_row(
            "select id, user_id, created_at, expires_at, used_at
             from magic_links
             where token_hash = ?",
            params![token_hash],
            |r| {
                Ok(MagicLinkRow {
                    id: r.get(0)?,
                    user_id: r.get(1)?,
                    created_at: r.get(2)?,
                    expires_at: r.get(3)?,
                    used_at: r.get(4)?,
                })
            },
        )
        .optional()
        .map_err(|e| ServerError::DbError(format!("select magic link failed: {e}")))?;

    Ok(row)
}

/// Consume a magic link token hash:
/// - must exist
/// - must be unexpired (expires_at > now)
/// - must be unused (used_at is null)
/// If valid, sets used_at=now and returns Some(user_id). Otherwise returns Ok(None).
///
/// Uses a transaction to prevent double-use races.
pub fn consume_magic_link(
    conn: &mut Connection,
    token_hash: &[u8],
    now: i64,
) -> Result<Option<i64>, ServerError> {
    let tx = conn
        .transaction()
        .map_err(|e| ServerError::DbError(format!("begin tx failed: {e}")))?;

    let row: Option<MagicLinkRow> = tx
        .query_row(
            "select id, user_id, created_at, expires_at, used_at
             from magic_links
             where token_hash = ?",
            params![token_hash],
            |r| {
                Ok(MagicLinkRow {
                    id: r.get(0)?,
                    user_id: r.get(1)?,
                    created_at: r.get(2)?,
                    expires_at: r.get(3)?,
                    used_at: r.get(4)?,
                })
            },
        )
        .optional()
        .map_err(|e| ServerError::DbError(format!("select magic link in tx failed: {e}")))?;

    let Some(ml) = row else {
        tx.rollback().ok();
        return Ok(None);
    };

    // Validate
    if ml.used_at.is_some() || ml.expires_at <= now {
        tx.rollback().ok();
        return Ok(None);
    }

    // Mark used (guard used_at IS NULL so only one consumer wins)
    let updated = tx
        .execute(
            "update magic_links set used_at = ?
             where id = ? and used_at is null",
            params![now, ml.id],
        )
        .map_err(|e| ServerError::DbError(format!("update magic link used_at failed: {e}")))?;

    if updated != 1 {
        tx.rollback().ok();
        return Ok(None);
    }

    tx.commit()
        .map_err(|e| ServerError::DbError(format!("commit tx failed: {e}")))?;

    Ok(Some(ml.user_id))
}

// TODO: Could move entitlements to own file.
#[derive(Debug, Clone)]
pub struct EntitlementInfo {
    pub plan_code: String,
    pub plan_name: String,
    pub download_limit: Option<i64>,
}

pub fn get_entitlement_info(
    conn: &rusqlite::Connection,
    user_id: i64,
) -> Result<EntitlementInfo, crate::errors::ServerError> {
    conn.query_row(
        r#"
        select e.plan_code, p.name, p.download_limit
        from entitlements e
        join plans p on p.code = e.plan_code
        where e.user_id = ?
        "#,
        rusqlite::params![user_id],
        |r| {
            Ok(EntitlementInfo {
                plan_code: r.get(0)?,
                plan_name: r.get(1)?,
                download_limit: r.get(2)?,
            })
        },
    )
    .map_err(|e| crate::errors::ServerError::DbError(format!("select entitlement failed: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply_schema(conn: &Connection) {
        conn.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;

            create table if not exists users (
              id            integer primary key,
              email         text not null unique,
              created_at    integer not null,
              last_login_at integer
            );

            create table if not exists magic_links (
              id          integer primary key,
              user_id     integer not null,
              token_hash  blob not null,
              created_at  integer not null,
              expires_at  integer not null,
              used_at     integer,
              foreign key(user_id) references users(id) on delete cascade
            );

            create index if not exists idx_magic_links_hash on magic_links(token_hash);
            create index if not exists idx_magic_links_user on magic_links(user_id);

            create table if not exists plans (
              id             integer primary key,
              code           text not null unique,
              name           text not null,
              price_cents    integer not null default 0,
              download_limit integer,
              trial_days     integer not null default 0,
              limit_window   text not null default 'month'
            );

            create table if not exists entitlements (
              id         integer primary key,
              user_id    integer not null unique,
              plan_code  text not null,
              granted_at integer not null,
              foreign key(user_id) references users(id) on delete cascade,
              foreign key(plan_code) references plans(code)
            );

            create index if not exists idx_entitlements_user on entitlements(user_id);
            create index if not exists idx_entitlements_plan on entitlements(plan_code);

            insert or ignore into plans (code, name, price_cents, download_limit, trial_days, limit_window)
            values
              ('free', 'Free', 0, 4, 0, 'month'),
              ('lifetime', 'Lifetime', 1900, null, 0, 'month');
            "#,
        )
        .unwrap();
    }

    #[test]
    fn get_or_create_user_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        apply_schema(&conn);

        let now = 1000;
        let id1 = get_or_create_user(&conn, "test@example.com", now).unwrap();
        let id2 = get_or_create_user(&conn, "test@example.com", now + 1).unwrap();
        assert_eq!(id1, id2);
    }

    #[test]
    fn ensure_entitlement_inserts_once() {
        let conn = Connection::open_in_memory().unwrap();
        apply_schema(&conn);

        let now = 1000;
        let user_id = get_or_create_user(&conn, "a@b.com", now).unwrap();

        ensure_entitlement(&conn, user_id, "free", now).unwrap();
        ensure_entitlement(&conn, user_id, "free", now + 10).unwrap(); // should not duplicate

        let count: i64 = conn
            .query_row(
                "select count(*) from entitlements where user_id = ?",
                params![user_id],
                |r| r.get(0),
            )
            .unwrap();

        assert_eq!(count, 1);
    }

    #[test]
    fn magic_link_insert_and_consume_once() {
        let mut conn = Connection::open_in_memory().unwrap();
        apply_schema(&conn);

        let now = 1000;
        let user_id = get_or_create_user(&conn, "c@d.com", now).unwrap();
        ensure_entitlement(&conn, user_id, "free", now).unwrap();

        let token_hash = b"fake_hash_32_bytes_len__________"; // just test bytes
        insert_magic_link(&conn, user_id, token_hash, now, now + 900).unwrap();

        let ok = consume_magic_link(&mut conn, token_hash, now + 1).unwrap();
        assert_eq!(ok, Some(user_id));

        // second consume should fail (used)
        let second = consume_magic_link(&mut conn, token_hash, now + 2).unwrap();
        assert_eq!(second, None);
    }

    #[test]
    fn magic_link_expired_cannot_be_consumed() {
        let mut conn = Connection::open_in_memory().unwrap();
        apply_schema(&conn);

        let now = 1000;
        let user_id = get_or_create_user(&conn, "e@f.com", now).unwrap();

        let token_hash = b"another_fake_hash______________";
        insert_magic_link(&conn, user_id, token_hash, now, now + 10).unwrap();

        // after expiry
        let res = consume_magic_link(&mut conn, token_hash, now + 11).unwrap();
        assert_eq!(res, None);
    }
}
