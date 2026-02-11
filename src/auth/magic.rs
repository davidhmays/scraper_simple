// src/auth/magic.rs
use crate::errors::ServerError;
use rusqlite::Connection;
use std::time::Duration;

use crate::auth::token::{generate_token_default, hash_token};
use crate::db::auth as db_auth;

#[derive(Debug, Clone)]
pub struct MagicLinkConfig {
    /// TTL for magic links in seconds.
    pub ttl_secs: i64,
    /// Relative path used when building links.
    /// Example: "/auth/magic"
    pub magic_path: String,
    /// Plan code to ensure on first request (e.g. "free").
    pub default_plan_code: String,
}

impl Default for MagicLinkConfig {
    fn default() -> Self {
        Self {
            ttl_secs: 15 * 60,
            magic_path: "/auth/magic".to_string(),
            default_plan_code: "free".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IssuedMagicLink {
    pub email: String,
    pub user_id: i64,
    /// Raw token (never store this in DB).
    pub token: String,
    pub expires_at: i64,
    /// Relative URL like "/auth/magic?token=..."
    pub link: String,
}

#[derive(Debug, Clone)]
pub struct RedeemedMagicLink {
    pub user_id: i64,
    pub email: String,
}

pub struct MagicLinkService {
    cfg: MagicLinkConfig,
}

impl MagicLinkService {
    pub fn new(cfg: MagicLinkConfig) -> Self {
        Self { cfg }
    }

    /// Trim + lowercase, minimal sanity check.
    pub fn normalize_email(email: &str) -> Result<String, ServerError> {
        let e = email.trim().to_lowercase();
        if e.is_empty() || !e.contains('@') || e.starts_with('@') || e.ends_with('@') {
            return Err(ServerError::BadRequest("invalid email".into()));
        }
        Ok(e)
    }

    fn build_link(&self, token: &str) -> String {
        format!("{}?token={}", self.cfg.magic_path, token)
    }

    /// Request a magic link (signup + login unified):
    /// - normalize email
    /// - get_or_create_user
    /// - ensure entitlement (default plan)
    /// - insert magic link (store hash only)
    ///
    /// Email sending is later: caller can log `issued.link`.
    pub fn request_link(
        &self,
        conn: &Connection,
        email: &str,
        now: i64,
    ) -> Result<IssuedMagicLink, ServerError> {
        let email = Self::normalize_email(email)?;
        let user_id = db_auth::get_or_create_user(conn, &email, now)?;

        // Ensure baseline entitlement exists.
        db_auth::ensure_entitlement(conn, user_id, &self.cfg.default_plan_code, now)?;

        let token = generate_token_default();
        let token_hash = hash_token(&token);
        let expires_at = now + self.cfg.ttl_secs;

        db_auth::insert_magic_link(conn, user_id, &token_hash, now, expires_at)?;

        Ok(IssuedMagicLink {
            email,
            user_id,
            token: token.clone(),
            expires_at,
            link: self.build_link(&token),
        })
    }

    /// Redeem a magic link:
    /// - hash token
    /// - consume_magic_link (transactional single-use)
    /// - return user_id (+ email for convenience)
    pub fn redeem(
        &self,
        conn: &mut Connection,
        token: &str,
        now: i64,
    ) -> Result<RedeemedMagicLink, ServerError> {
        let token = token.trim();
        if token.is_empty() {
            return Err(ServerError::BadRequest("missing token".into()));
        }

        let token_hash = hash_token(token);
        let Some(user_id) = db_auth::consume_magic_link(conn, &token_hash, now)? else {
            return Err(ServerError::Unauthorized("invalid or expired link".into()));
        };

        // Useful for logging + sessions later.
        let email: String = conn
            .query_row(
                "select email from users where id = ?",
                rusqlite::params![user_id],
                |r| r.get(0),
            )
            .map_err(|e| ServerError::DbError(format!("select user email failed: {e}")))?;

        Ok(RedeemedMagicLink { user_id, email })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::{params, Connection};

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

    fn svc() -> MagicLinkService {
        MagicLinkService::new(MagicLinkConfig {
            ttl_secs: 60, // keep short for tests
            magic_path: "/auth/magic".to_string(),
            default_plan_code: "free".to_string(),
        })
    }

    #[test]
    fn normalize_email_trims_and_lowercases() {
        let e = MagicLinkService::normalize_email("  Test@Example.COM ").unwrap();
        assert_eq!(e, "test@example.com");
    }

    #[test]
    fn normalize_email_rejects_invalid() {
        assert!(MagicLinkService::normalize_email("").is_err());
        assert!(MagicLinkService::normalize_email("no-at-symbol").is_err());
        assert!(MagicLinkService::normalize_email("@example.com").is_err());
        assert!(MagicLinkService::normalize_email("test@").is_err());
    }

    #[test]
    fn request_link_creates_user_entitlement_and_magic_link() {
        let conn = Connection::open_in_memory().unwrap();
        apply_schema(&conn);
        let service = svc();

        let now = 1000;
        let issued = service
            .request_link(&conn, "User@Example.com", now)
            .unwrap();

        // user exists
        let user_id: i64 = conn
            .query_row(
                "select id from users where email = ?",
                params!["user@example.com"],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(issued.user_id, user_id);

        // entitlement exists
        let plan_code: String = conn
            .query_row(
                "select plan_code from entitlements where user_id = ?",
                params![user_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(plan_code, "free");

        // magic link exists and matches hash
        let expected_hash = hash_token(&issued.token);
        let token_hash: Vec<u8> = conn
            .query_row(
                "select token_hash from magic_links where user_id = ? order by id desc limit 1",
                params![user_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(token_hash.as_slice(), expected_hash.as_slice());

        // link format
        assert!(issued.link.starts_with("/auth/magic?token="));
        assert!(issued.link.contains(&issued.token));
        assert_eq!(issued.expires_at, now + 60);
    }

    #[test]
    fn redeem_succeeds_once_then_fails() {
        let mut conn = Connection::open_in_memory().unwrap();
        apply_schema(&conn);
        let service = svc();

        let now = 1000;
        let issued = service.request_link(&conn, "a@b.com", now).unwrap();

        // redeem once
        let redeemed = service.redeem(&mut conn, &issued.token, now + 1).unwrap();
        assert_eq!(redeemed.user_id, issued.user_id);
        assert_eq!(redeemed.email, "a@b.com");

        // redeem twice should fail (used)
        let second = service.redeem(&mut conn, &issued.token, now + 2);
        match second {
            Err(ServerError::Unauthorized(_)) => {}
            other => panic!("expected Unauthorized, got: {:?}", other),
        }
    }

    #[test]
    fn redeem_fails_if_expired() {
        let mut conn = Connection::open_in_memory().unwrap();
        apply_schema(&conn);

        let service = MagicLinkService::new(MagicLinkConfig {
            ttl_secs: 1,
            magic_path: "/auth/magic".to_string(),
            default_plan_code: "free".to_string(),
        });

        let now = 1000;
        let issued = service.request_link(&conn, "x@y.com", now).unwrap();

        // at now+2 it's expired (expires_at = now+1, consume checks expires_at <= now)
        let res = service.redeem(&mut conn, &issued.token, now + 2);
        match res {
            Err(ServerError::Unauthorized(_)) => {}
            other => panic!("expected Unauthorized, got: {:?}", other),
        }
    }

    #[test]
    fn redeem_rejects_missing_token() {
        let mut conn = Connection::open_in_memory().unwrap();
        apply_schema(&conn);
        let service = svc();

        let res = service.redeem(&mut conn, "   ", 1000);
        match res {
            Err(ServerError::BadRequest(_)) => {}
            other => panic!("expected BadRequest, got: {:?}", other),
        }
    }
}
