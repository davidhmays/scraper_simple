use crate::auth::sessions;
use crate::db::downloads::record_download;
use crate::db::magic_auth::{redeem_magic_link, request_magic_link};
use crate::router::handle;
use crate::tests::utils::init_test_db;
use astra::Body;
use http::{Method, Request};
use rusqlite::params;
use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn create_authenticated_user(db: &crate::db::connection::Database) -> (i64, String) {
    let now = now_unix();
    let email = "admin@example.com";

    // 1. Request magic link
    let issued = request_magic_link(db, email, now).expect("Failed to request link");

    // 2. Redeem
    let redeemed = redeem_magic_link(db, &issued.token, now).expect("Failed to redeem");

    // 3. Create Session
    let token = db
        .with_conn(|conn| sessions::create_session(conn, redeemed.user_id, now))
        .expect("Failed to create session");

    // Promote to admin
    db.with_conn(|conn| {
        conn.execute(
            "update users set is_admin = 1 where id = ?",
            params![redeemed.user_id],
        )
        .map_err(|e| crate::errors::ServerError::DbError(e.to_string()))
    })
    .expect("Failed to promote to admin");

    (redeemed.user_id, token)
}

#[test]
fn admin_page_loads_for_authenticated_user() {
    // Note: currently all users are admins in dev mode
    let db = init_test_db();
    let (_, session_token) = create_authenticated_user(&db);

    let req = Request::builder()
        .method(Method::GET)
        .uri("/admin")
        .header("Cookie", format!("session={}", session_token))
        .body(Body::empty())
        .unwrap();

    let resp = handle(req, &db).expect("Handler failed");

    assert_eq!(resp.status(), 200, "Admin page should load");

    let mut body = String::new();
    resp.into_body().reader().read_to_string(&mut body).unwrap();

    assert!(body.contains("Admin Dashboard"));
    assert!(body.contains("admin@example.com")); // User should be in the table
}

#[test]
fn admin_can_reset_usage() {
    let db = init_test_db();
    let now = now_unix();
    let (user_id, session_token) = create_authenticated_user(&db);

    // 1. Record some usage
    db.with_conn(|conn| record_download(conn, user_id, "UT", now))
        .expect("Failed to record usage");

    // Verify usage is 1
    let usage = db
        .with_conn(|conn| crate::db::downloads::count_downloads_this_month(conn, user_id, now))
        .unwrap();
    assert_eq!(usage, 1, "Usage should be 1 before reset");

    // 2. Reset via API
    let req = Request::builder()
        .method(Method::POST)
        .uri(format!("/admin/users/{}/reset-usage", user_id))
        .header("Cookie", format!("session={}", session_token))
        .body(Body::empty())
        .unwrap();

    let resp = handle(req, &db).expect("Handler failed");

    assert_eq!(resp.status(), 302, "Should redirect after reset");
    assert_eq!(
        resp.headers().get("Location").unwrap().to_str().unwrap(),
        "/admin"
    );

    // 3. Verify usage is 0
    let usage_after = db
        .with_conn(|conn| crate::db::downloads::count_downloads_this_month(conn, user_id, now))
        .unwrap();
    assert_eq!(usage_after, 0, "Usage should be 0 after reset");
}
