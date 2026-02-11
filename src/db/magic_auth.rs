use rusqlite::params;

use crate::auth::magic::{IssuedMagicLink, MagicLinkConfig, MagicLinkService, RedeemedMagicLink};
use crate::db::connection::Database;
use crate::errors::ServerError;

/// Request a magic link: creates user, ensures entitlement, inserts magic link.
/// Returns the issued link (raw token included so caller can email/log).
pub fn request_magic_link(
    db: &Database,
    email: &str,
    now: i64,
) -> Result<IssuedMagicLink, ServerError> {
    let svc = MagicLinkService::new(MagicLinkConfig::default());
    db.with_conn(|conn| svc.request_link(conn, email, now))
}

/// Redeem a magic link token (single-use), updates last_login_at, and returns user info.
/// Sessions come next.
pub fn redeem_magic_link(
    db: &Database,
    token: &str,
    now: i64,
) -> Result<RedeemedMagicLink, ServerError> {
    let svc = MagicLinkService::new(MagicLinkConfig::default());

    db.with_conn(|conn| {
        let redeemed = svc.redeem(conn, token, now)?;

        conn.execute(
            "update users set last_login_at = ? where id = ?",
            params![now, redeemed.user_id],
        )
        .map_err(|e| ServerError::DbError(format!("update last_login_at failed: {e}")))?;

        Ok(redeemed)
    })
}
