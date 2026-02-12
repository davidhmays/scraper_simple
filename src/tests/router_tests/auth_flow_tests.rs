use crate::router::handle;
use crate::tests::utils::init_test_db;
use astra::Body;
use http::{Method, Request};
use std::io::Read;

#[test]
fn login_page_loads_successfully() {
    let db = init_test_db();

    let req = Request::builder()
        .method(Method::GET)
        .uri("/login")
        .body(Body::empty())
        .unwrap();

    let resp = handle(req, &db).expect("Failed to handle request");

    assert_eq!(resp.status(), 200);

    let mut body = String::new();
    resp.into_body().reader().read_to_string(&mut body).unwrap();

    assert!(body.contains("Sign in"));
    assert!(body.contains("form"));
}

#[test]
fn request_link_returns_partial_html_for_htmx() {
    let db = init_test_db();
    let email = "test@example.com";
    let body_data = format!("email={}", email);

    let req = Request::builder()
        .method(Method::POST)
        .uri("/auth/request-link")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(Body::from(body_data.as_bytes().to_vec()))
        .unwrap();

    let resp = handle(req, &db).expect("Failed to handle request");

    assert_eq!(resp.status(), 200);

    let mut body = String::new();
    resp.into_body().reader().read_to_string(&mut body).unwrap();

    // Verify success message
    assert!(body.contains("Check your email"));
    assert!(body.contains(email));

    // Verify it is a partial (no full html structure), which is crucial for HTMX swapping
    assert!(!body.contains("<!DOCTYPE html>"));
    assert!(!body.contains("<html"));
}
