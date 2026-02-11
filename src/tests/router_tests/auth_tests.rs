// src/tests/auth_tests.rs
use crate::auth::magic::{MagicLinkConfig, MagicLinkService};
use crate::db::{connection::init_db, connection::Database};
use crate::errors::ServerError;
use crate::router::handle; // your request handler
use astra::{Body, Request};
use http::Method;
use std::time::{SystemTime, UNIX_EPOCH};

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

/// Initialize a fresh DB for testing
fn setup_db() -> Database {
    let db = Database::new("test_auth.sqlite");
    init_db(&db, "sql/schema.sql").expect("Failed to initialize DB");
    db
}

#[test]
fn get_magic_consumes_link_and_redirects() -> Result<(), Box<dyn std::error::Error>> {
    let db = setup_db();

    // Issue a magic link
    let token = db.with_conn(|conn| -> Result<String, ServerError> {
        let svc = MagicLinkService::new(MagicLinkConfig::default());
        let issued = svc
            .request_link(conn, "c@d.com", now_unix())
            .map_err(|e| ServerError::DbError(format!("magic link request failed: {e}")))?;
        Ok(issued.token)
    })?;

    let mut req = Request::new(Body::empty());

    // Set the HTTP method correctly
    *req.method_mut() = Method::GET;

    // Set the URI
    *req.uri_mut() = format!("/auth/magic?token={}", token).parse().unwrap();

    // Call router handler
    let resp = handle(req, &db)?;

    // Expect redirect to dashboard
    assert_eq!(resp.status(), 302);
    let loc = resp
        .headers()
        .get("Location")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert_eq!(loc, "/dashboard");

    // Check DB state
    db.with_conn(|conn| {
        let used_count: i64 = conn
            .query_row(
                "select count(*) from magic_links where used_at is not null",
                [],
                |r| r.get(0),
            )
            .map_err(|e| ServerError::DbError(format!("query magic_links failed: {e}")))?;
        assert!(used_count >= 1);
        Ok(())
    })?;

    Ok(())
}
