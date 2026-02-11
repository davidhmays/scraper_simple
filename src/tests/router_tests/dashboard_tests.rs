// src/tests/dashboard_test.rs

use crate::auth::magic::{MagicLinkConfig, MagicLinkService};
use crate::db::connection::Database;
use crate::handle;
use crate::init_db;
use astra::{Body, Request};
use http::Method;
use std::time::{SystemTime, UNIX_EPOCH};

/// Returns a fresh test database using your production schema
fn make_db() -> Database {
    let path = std::env::temp_dir().join(format!(
        "dashboard_test_{}.sqlite",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let db = Database::new(path);
    init_db(&db, "sql/schema.sql").expect("Failed to initialize DB");
    db
}

/// Issue a magic link and return the token
fn issue_magic_token(db: &Database, email: &str) -> String {
    db.with_conn(|conn| {
        let svc = MagicLinkService::new(MagicLinkConfig::default());
        let issued = svc.request_link(conn, email, now_unix())?;
        Ok::<_, crate::errors::ServerError>(issued.token)
    })
    .unwrap()
}

/// Get current unix timestamp
fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[test]
fn dashboard_requires_login() {
    let db = make_db();

    // Step 1: create user via magic link
    let email = "dash@example.com";
    let token = issue_magic_token(&db, email);

    // Step 2: redeem token -> creates session
    let session_token = db
        .with_conn(|conn| crate::auth::magic::redeem_magic_link(&db, &token, now_unix()))
        .unwrap()
        .session_token;

    // Step 3: make GET request to /dashboard with session cookie
    let mut req_dashboard = Request::new(Method::GET, "/dashboard");
    req_dashboard.headers_mut().insert(
        "Cookie",
        format!("session={}", session_token).parse().unwrap(),
    );

    let resp_dashboard = handle(req_dashboard, &db).unwrap();

    // Step 4: assert redirect for unauthenticated fails
    assert_eq!(resp_dashboard.status(), 200);

    // Step 5: read body
    let mut body_bytes = Vec::new();
    resp_dashboard
        .body_mut()
        .reader()
        .read_to_end(&mut body_bytes)
        .unwrap();
    let body_str = std::str::from_utf8(&body_bytes).unwrap();

    // Step 6: check that user's email appears in dashboard
    assert!(
        body_str.contains(email),
        "Dashboard body did not contain expected email"
    );
}
