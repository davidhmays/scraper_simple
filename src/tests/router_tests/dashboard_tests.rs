// src/tests/router_tests/dashboard_tests.rs

use crate::auth::sessions;
use crate::db::connection::{init_db, Database};
use crate::db::magic_auth::{redeem_magic_link, request_magic_link};
use crate::router::handle;
use astra::Body;
use http::{Method, Request};
use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

fn tmp_db_path(name: &str) -> String {
    let mut p = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    p.push(format!("{}_{}.sqlite", name, nanos));
    p.to_string_lossy().to_string()
}

fn make_db() -> Database {
    let path = tmp_db_path("dashboard_test");
    let db = Database::new(path);
    init_db(&db, "sql/schema.sql").expect("Failed to initialize DB");
    db
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[test]
fn dashboard_accessible_with_valid_session() {
    let db = make_db();
    let email = "dashboard_user@example.com";
    let now = now_unix();

    // 1. Request magic link (creates user & entitlement)
    let issued = request_magic_link(&db, email, now).expect("Failed to request link");

    // 2. Redeem magic link (updates last_login_at)
    let redeemed = redeem_magic_link(&db, &issued.token, now).expect("Failed to redeem link");

    // 3. Create session manually (simulating router behavior)
    let session_token = db
        .with_conn(|conn| sessions::create_session(conn, redeemed.user_id, now))
        .expect("Failed to create session");

    // 4. Make request to /dashboard with cookie
    let req = Request::builder()
        .method(Method::GET)
        .uri("/dashboard")
        .header("Cookie", format!("session={}", session_token))
        .body(Body::empty())
        .unwrap();

    let mut resp = handle(req, &db).expect("Handler failed");

    // 5. Verify response
    assert_eq!(resp.status(), 200, "Dashboard should return 200 OK");

    let mut body = String::new();
    resp.body_mut().reader().read_to_string(&mut body).unwrap();

    // Check for user email in the dashboard (it's usually displayed)
    assert!(body.contains(email), "Dashboard should contain user email");
}

#[test]
fn dashboard_redirects_without_session() {
    let db = make_db();

    let req = Request::builder()
        .method(Method::GET)
        .uri("/dashboard")
        .body(Body::empty())
        .unwrap();

    let resp = handle(req, &db).expect("Handler failed");

    assert_eq!(
        resp.status(),
        302,
        "Dashboard should redirect if not logged in"
    );
    let location = resp.headers().get("Location").unwrap().to_str().unwrap();
    assert_eq!(location, "/", "Should redirect to home");
}
