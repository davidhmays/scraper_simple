use crate::db::connection::Database;
use crate::db::listings::get_counties_by_state;
use crate::db::listings::get_listings_by_state;

use crate::errors::ServerError;
use crate::mailings::BrevoMailer;
use crate::responses::{html_response, ResultResp};
use crate::scraper::RealtorScraper;
use crate::spreadsheets::export_listings_xlsx;

use crate::templates;
use crate::templates::pages::admin::AdminVm;
use crate::templates::pages::dashboard::DashboardVm;
use crate::templates::pages::preview::preview_table;

use astra::{Body, Request, ResponseBuilder};
use maud::html;
use rusqlite::params;
use std::io::Read;
use url::form_urlencoded;

use crate::auth::sessions;
use crate::db::magic_auth::{redeem_magic_link, request_magic_link};

use std::time::{SystemTime, UNIX_EPOCH};

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn get_cookie(req: &Request, name: &str) -> Option<String> {
    let cookie = req
        .headers()
        .get("Cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    cookie
        .split(';')
        .find_map(|part| {
            let part = part.trim();
            part.strip_prefix(&format!("{}=", name))
        })
        .map(|s| s.to_string())
}

fn current_user(
    req: &Request,
    db: &Database,
    now: i64,
) -> Result<Option<(i64, String)>, ServerError> {
    let cookie = req
        .headers()
        .get("Cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let session_token = cookie.split(';').find_map(|part| {
        let part = part.trim();
        part.strip_prefix("session=")
    });

    let Some(raw_token) = session_token else {
        return Ok(None);
    };

    db.with_conn(|conn| crate::auth::sessions::load_user_from_session(conn, raw_token, now))
        .map_err(|e| ServerError::DbError(e.to_string()))
}

fn query_param(req: &Request, key: &str) -> Option<String> {
    req.uri().query().and_then(|q| {
        for (k, v) in url::form_urlencoded::parse(q.as_bytes()) {
            if k == key {
                return Some(v.into_owned());
            }
        }
        None
    })
}

fn body_to_bytes(req: &mut Request) -> Result<Vec<u8>, ServerError> {
    let mut out = Vec::new();
    req.body_mut()
        .reader()
        .read_to_end(&mut out)
        .map_err(|e| ServerError::BadRequest(format!("Failed to read request body: {e}")))?;
    Ok(out)
}

fn form_first(pairs: &[(String, String)], key: &str) -> Option<String> {
    pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
}

fn fetch_dashboard_vm(
    conn: &rusqlite::Connection,
    user_id: i64,
    email: String,
    now: i64,
    last_state: Option<String>,
) -> Result<DashboardVm, ServerError> {
    let plan_info = crate::db::plans::get_user_plan(conn, user_id)
        .map_err(|e| ServerError::DbError(e.to_string()))?;

    let usage = crate::db::downloads::count_downloads_this_month(conn, user_id, now)
        .map_err(|e| ServerError::DbError(e.to_string()))?;

    let is_admin = crate::db::users::is_user_admin(conn, user_id)
        .map_err(|e| ServerError::DbError(e.to_string()))?;

    Ok(DashboardVm {
        email,
        plan_code: plan_info.code,
        plan_name: plan_info.name,
        download_limit: plan_info.download_limit,
        usage,
        is_admin,
        last_state,
    })
}

pub fn handle(mut req: Request, db: &Database) -> ResultResp {
    // Clone path parts to avoid borrow checker issues with mutable body reading
    let method = req.method().as_str().to_string();
    let path = req.uri().path().to_string();

    match (method.as_str(), path.as_str()) {
        ("GET", path) if path.starts_with("/static") => serve_static(path),
        ("GET", "/") => html_response(templates::pages::home_page()),
        ("GET", "/login") => html_response(templates::pages::login::login_page()),

        ("GET", "/admin") => {
            let now = now_unix();

            // 1. Authenticate
            let user = current_user(&req, db, now)?;
            let Some((user_id, _)) = user else {
                return Ok(ResponseBuilder::new()
                    .status(302)
                    .header("Location", "/login")
                    .body(Body::empty())
                    .unwrap());
            };

            // 2. Check Admin
            let is_admin = db.with_conn(|conn| crate::db::users::is_user_admin(conn, user_id))?;

            if !is_admin {
                return Err(ServerError::Unauthorized("Admin access required".into()));
            }

            // 3. Fetch Data
            let users =
                db.with_conn(|conn| crate::db::users::get_all_users_with_stats(conn, now))?;
            let plans = db.with_conn(|conn| crate::db::plans::get_all_plans(conn))?;
            let scrapes = db.with_conn(|conn| crate::db::scrapes::get_recent_scrapes(conn))?;

            html_response(templates::pages::admin_page(&AdminVm {
                users,
                plans,
                scrapes,
            }))
        }

        ("POST", "/admin/scrape") => {
            let now = now_unix();

            // 1. Authenticate
            let user = current_user(&req, db, now)?;
            let Some((user_id, _)) = user else {
                return Ok(ResponseBuilder::new()
                    .status(302)
                    .header("Location", "/login")
                    .body(Body::empty())
                    .unwrap());
            };

            // 2. Check Admin
            let is_admin = db.with_conn(|conn| crate::db::users::is_user_admin(conn, user_id))?;

            if !is_admin {
                return Err(ServerError::Unauthorized("Admin access required".into()));
            }

            // 3. Parse State
            let body_bytes = body_to_bytes(&mut req)?;
            let pairs: Vec<(String, String)> =
                form_urlencoded::parse(&body_bytes).into_owned().collect();
            let state_abbr = form_first(&pairs, "state")
                .ok_or_else(|| ServerError::BadRequest("state is required".into()))?;

            // 4. Lookup Full Name
            let state_name = crate::geos::US_STATES
                .iter()
                .find(|(abbr, _)| *abbr == state_abbr)
                .map(|(_, name)| name.to_string())
                .ok_or_else(|| ServerError::BadRequest("Invalid state".into()))?;

            // 5. Start Scrape
            let db_clone = db.clone();
            std::thread::spawn(move || {
                RealtorScraper::run_realtor_scrape(&db_clone, state_name, state_abbr);
            });

            Ok(ResponseBuilder::new()
                .status(302)
                .header("Location", "/admin")
                .body(Body::empty())
                .unwrap())
        }

        ("POST", path) if path.starts_with("/admin/users/") && path.ends_with("/reset-usage") => {
            let now = now_unix();

            let user = current_user(&req, db, now)?;
            let Some((user_id, _)) = user else {
                return Ok(ResponseBuilder::new()
                    .status(302)
                    .header("Location", "/login")
                    .body(Body::empty())
                    .unwrap());
            };

            let is_admin = db.with_conn(|conn| crate::db::users::is_user_admin(conn, user_id))?;
            if !is_admin {
                return Err(ServerError::Unauthorized("Admin access required".into()));
            }

            let parts: Vec<&str> = path.split('/').collect();
            let target_id = parts
                .get(3)
                .and_then(|s| s.parse::<i64>().ok())
                .ok_or(ServerError::BadRequest("Invalid user id".into()))?;

            db.with_conn(|conn| crate::db::downloads::reset_user_downloads(conn, target_id, now))?;

            Ok(ResponseBuilder::new()
                .status(302)
                .header("Location", "/admin")
                .body(Body::empty())
                .unwrap())
        }

        ("POST", path) if path.starts_with("/admin/plans/") && path.ends_with("/limit") => {
            let now = now_unix();

            let user = current_user(&req, db, now)?;
            let Some((user_id, _)) = user else {
                return Ok(ResponseBuilder::new()
                    .status(302)
                    .header("Location", "/login")
                    .body(Body::empty())
                    .unwrap());
            };

            let is_admin = db.with_conn(|conn| crate::db::users::is_user_admin(conn, user_id))?;
            if !is_admin {
                return Err(ServerError::Unauthorized("Admin access required".into()));
            }

            let parts: Vec<&str> = path.split('/').collect();
            let code = parts.get(3).unwrap_or(&"");

            let body_bytes = body_to_bytes(&mut req)?;
            let pairs: Vec<(String, String)> =
                form_urlencoded::parse(&body_bytes).into_owned().collect();
            let limit_str = form_first(&pairs, "limit")
                .ok_or_else(|| ServerError::BadRequest("limit is required".into()))?;
            let limit = limit_str
                .parse::<i64>()
                .map_err(|_| ServerError::BadRequest("invalid limit".into()))?;

            db.with_conn(|conn| crate::db::plans::update_plan_limit(conn, code, Some(limit)))?;

            Ok(ResponseBuilder::new()
                .status(302)
                .header("Location", "/admin")
                .body(Body::empty())
                .unwrap())
        }

        ("GET", "/campaigns") => {
            let mut state = "UT".to_string();
            if let Some(q) = req.uri().query() {
                for (k, v) in url::form_urlencoded::parse(q.as_bytes()) {
                    if k == "state" {
                        state = v.to_string().to_uppercase();
                    }
                }
            }
            let counties = get_counties_by_state(db, &state)?;
            html_response(templates::pages::campaigns_page(&state, &counties))
        }

        // Support for /export?state=XX (Form submission)
        ("GET", "/export") => {
            let now = now_unix();

            // 1. Authenticate
            let user = current_user(&req, db, now)?;
            let Some((user_id, _)) = user else {
                return Ok(ResponseBuilder::new()
                    .status(302)
                    .header("Location", "/login")
                    .body(Body::empty())
                    .unwrap());
            };

            // 2. Check limits
            let allowed = db.with_conn(|conn| {
                let plan = crate::db::plans::get_user_plan(conn, user_id)?;
                if let Some(limit) = plan.download_limit {
                    let usage =
                        crate::db::downloads::count_downloads_this_month(conn, user_id, now)?;
                    if usage >= limit {
                        return Ok(false);
                    }
                }
                Ok(true)
            })?;

            if !allowed {
                return Err(ServerError::BadRequest(
                    "Download limit reached for this month.".into(),
                ));
            }

            let state = query_param(&req, "state")
                .ok_or_else(|| ServerError::BadRequest("state is required".into()))?
                .to_uppercase();

            let listings = get_listings_by_state(db, &state)?;
            let mut resp = export_listings_xlsx(&listings, &state)?;

            // 3. Record Download
            db.with_conn(|conn| crate::db::downloads::record_download(conn, user_id, &state, now))?;

            resp.headers_mut().insert(
                "Set-Cookie",
                format!(
                    "last_state={}; Max-Age=31536000; SameSite=Lax; Path=/",
                    state
                )
                .parse()
                .unwrap(),
            );

            Ok(resp)
        }

        // Legacy /export/UT support (can share logic but keeping separate for now to minimize churn)
        ("GET", path) if path.starts_with("/export/") => {
            let state = path.trim_start_matches("/export/").to_uppercase();
            let listings = get_listings_by_state(db, &state)?;
            export_listings_xlsx(&listings, &state)
        }

        ("GET", "/dashboard") => {
            let now = now_unix();
            let user = current_user(&req, db, now)?;
            let Some((user_id, email)) = user else {
                return Ok(ResponseBuilder::new()
                    .status(302)
                    .header("Location", "/")
                    .body(Body::empty())
                    .unwrap());
            };

            let last_state = get_cookie(&req, "last_state");
            let dashboard_vm =
                db.with_conn(|conn| fetch_dashboard_vm(conn, user_id, email, now, last_state))?;

            html_response(templates::pages::dashboard_page(&dashboard_vm))
        }

        ("GET", "/dashboard/export-card") => {
            let now = now_unix();
            let user = current_user(&req, db, now)?;
            let Some((user_id, email)) = user else {
                return Ok(ResponseBuilder::new()
                    .status(302)
                    .header("Location", "/login")
                    .body(Body::empty())
                    .unwrap());
            };

            let last_state = get_cookie(&req, "last_state");
            let dashboard_vm =
                db.with_conn(|conn| fetch_dashboard_vm(conn, user_id, email, now, last_state))?;

            let mut resp =
                html_response(templates::pages::dashboard::export_card(&dashboard_vm)).unwrap();
            resp.headers_mut()
                .insert("Cache-Control", "no-store".parse().unwrap());
            Ok(resp)
        }

        ("GET", "/dashboard/preview") => {
            let now = now_unix();
            let user = current_user(&req, db, now)?;
            let Some((user_id, _)) = user else {
                return Ok(ResponseBuilder::new()
                    .status(302)
                    .header("Location", "/login")
                    .body(Body::empty())
                    .unwrap());
            };

            let state = query_param(&req, "state")
                .unwrap_or_default()
                .to_uppercase();
            if state.is_empty() {
                return html_response(html! {});
            }

            let listings = get_listings_by_state(db, &state)?;
            let total_count = listings.len();

            let is_paid = db.with_conn(|conn| {
                let plan = crate::db::plans::get_user_plan(conn, user_id)?;
                Ok(plan.download_limit.is_none())
            })?;

            html_response(preview_table(&listings, total_count, is_paid))
        }

        ("GET", "/auth/magic") => {
            let token = query_param(&req, "token")
                .ok_or_else(|| ServerError::BadRequest("missing token".into()))?;

            let now = now_unix();
            let redeemed = redeem_magic_link(db, &token, now)?;
            let session_token =
                db.with_conn(|conn| sessions::create_session(conn, redeemed.user_id, now))?;

            Ok(ResponseBuilder::new()
                .status(302)
                .header("Location", "/dashboard")
                .header(
                    "Set-Cookie",
                    format!("session={}; HttpOnly; SameSite=Lax; Path=/", session_token),
                )
                .body(Body::empty())
                .unwrap())
        }

        ("POST", "/auth/request-link") => {
            eprintln!("POST /auth/request-link");

            let body_bytes = body_to_bytes(&mut req)?;
            let pairs: Vec<(String, String)> =
                form_urlencoded::parse(&body_bytes).into_owned().collect();

            let email = form_first(&pairs, "email")
                .ok_or_else(|| ServerError::BadRequest("email is required".into()))?;

            let now = now_unix();
            let issued = request_magic_link(db, &email, now)?;

            eprintln!("🔐 MAGIC LINK for {} => {}", issued.email, issued.link);

            if let Ok(api_key) = std::env::var("BREVO_API_KEY") {
                let sender_email = std::env::var("SENDER_EMAIL")
                    .unwrap_or_else(|_| "noreply@scraper-simple.com".to_string());
                let sender_name =
                    std::env::var("SENDER_NAME").unwrap_or_else(|_| "Scraper Simple".to_string());

                let base_url = std::env::var("BASE_URL")
                    .unwrap_or_else(|_| "http://localhost:3000".to_string());
                let full_link = format!("{}{}", base_url, issued.link);

                let recipient_email = issued.email.clone();

                std::thread::spawn(move || {
                    let mailer = BrevoMailer::new(api_key, sender_email, sender_name);
                    match mailer.send_magic_link(&recipient_email, &full_link) {
                        Ok(_) => eprintln!("📧 Email sent to {}", recipient_email),
                        Err(e) => {
                            eprintln!("❌ Failed to send email to {}: {:?}", recipient_email, e)
                        }
                    }
                });
            } else {
                eprintln!("⚠️ BREVO_API_KEY not set. Email not sent.");
            }

            html_response(templates::pages::check_email_content(&email))
        }

        ("POST", "/checkout") => {
            let now = now_unix();
            let user = current_user(&req, db, now)?;
            let Some((_, email)) = user else {
                return Ok(ResponseBuilder::new()
                    .status(302)
                    .header("Location", "/login")
                    .body(Body::empty())
                    .unwrap());
            };

            let secret_key = std::env::var("STRIPE_SECRET_KEY").map_err(|_| {
                eprintln!("Missing STRIPE_SECRET_KEY");
                ServerError::InternalError
            })?;
            let price_id = std::env::var("STRIPE_PRICE_ID").map_err(|_| {
                eprintln!("Missing STRIPE_PRICE_ID");
                ServerError::InternalError
            })?;
            let base_url =
                std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

            let client = reqwest::blocking::Client::new();
            let params = [
                ("mode", "payment"),
                ("payment_method_types[0]", "card"),
                ("line_items[0][price]", &price_id),
                ("line_items[0][quantity]", "1"),
                ("customer_email", &email),
                (
                    "success_url",
                    &format!(
                        "{}/checkout/success?session_id={{CHECKOUT_SESSION_ID}}",
                        base_url
                    ),
                ),
                ("cancel_url", &format!("{}/dashboard", base_url)),
            ];

            let resp = client
                .post("https://api.stripe.com/v1/checkout/sessions")
                .basic_auth(&secret_key, None::<&str>)
                .form(&params)
                .send()
                .map_err(|e| ServerError::BadRequest(format!("Stripe request failed: {e}")))?;

            if !resp.status().is_success() {
                let text = resp.text().unwrap_or_default();
                eprintln!("Stripe error: {}", text);
                return Err(ServerError::BadRequest("Stripe checkout failed".into()));
            }

            let json: serde_json::Value = resp
                .json()
                .map_err(|e| ServerError::BadRequest(format!("Bad response from Stripe: {e}")))?;

            let url = json["url"]
                .as_str()
                .ok_or_else(|| ServerError::BadRequest("No checkout URL returned".into()))?;

            Ok(ResponseBuilder::new()
                .status(303)
                .header("Location", url)
                .body(Body::empty())
                .unwrap())
        }

        ("GET", "/checkout/success") => {
            let now = now_unix();
            let user = current_user(&req, db, now)?;
            let Some((user_id, _)) = user else {
                return Ok(ResponseBuilder::new()
                    .status(302)
                    .header("Location", "/login")
                    .body(Body::empty())
                    .unwrap());
            };

            let session_id = query_param(&req, "session_id")
                .ok_or_else(|| ServerError::BadRequest("Missing session_id".into()))?;

            let secret_key = std::env::var("STRIPE_SECRET_KEY").map_err(|_| {
                eprintln!("Missing STRIPE_SECRET_KEY");
                ServerError::InternalError
            })?;

            // Verify payment
            let client = reqwest::blocking::Client::new();
            let url = format!("https://api.stripe.com/v1/checkout/sessions/{}", session_id);

            let resp = client
                .get(&url)
                .basic_auth(&secret_key, None::<&str>)
                .send()
                .map_err(|e| ServerError::BadRequest(format!("Stripe verify failed: {e}")))?;

            let json: serde_json::Value = resp
                .json()
                .map_err(|_| ServerError::BadRequest("Bad json from Stripe".into()))?;

            if json.get("payment_status").and_then(|s| s.as_str()) == Some("paid") {
                // Upgrade User
                db.with_conn(|conn| {
                    crate::db::plans::upgrade_user_plan(conn, user_id, "lifetime", now)
                })?;
            } else {
                return Err(ServerError::BadRequest("Payment not verified".into()));
            }

            Ok(ResponseBuilder::new()
                .status(302)
                .header("Location", "/dashboard")
                .body(Body::empty())
                .unwrap())
        }

        _ => Err(ServerError::NotFound),
    }
}

pub fn serve_static(path: &str) -> ResultResp {
    let fs_path = &path[1..]; // strip leading "/"
    if fs_path.contains("..") {
        return Err(ServerError::BadRequest("Invalid path".into()));
    }

    let bytes = std::fs::read(fs_path).map_err(|_| ServerError::NotFound)?;
    let mime = mime_for(fs_path);

    let resp = ResponseBuilder::new()
        .status(200)
        .header("Content-Type", mime)
        .body(Body::from(bytes))
        .unwrap();

    Ok(resp)
}

fn mime_for(path: &str) -> &'static str {
    if path.ends_with(".css") {
        "text/css"
    } else if path.ends_with(".js") {
        "application/javascript"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        "image/jpeg"
    } else if path.ends_with(".gif") {
        "image/gif"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".html") {
        "text/html"
    } else if path.ends_with(".txt") {
        "text/plain"
    } else if path.ends_with(".ttf") {
        "font/ttf"
    } else {
        "application/octet-stream"
    }
}

#[cfg(test)]
mod router_auth_tests {
    use super::*;
    use crate::auth::magic::{MagicLinkConfig, MagicLinkService};
    use crate::db::connection::Database; // Ensure import

    use astra::{Body, Request};
    use http::{Method, Request as HttpRequest};

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

    fn make_db_with_schema() -> Database {
        let path = tmp_db_path("router_auth");
        let db = Database::new(path);

        db.with_conn(|conn| {
            conn.execute_batch(
                r#"
                PRAGMA foreign_keys = ON;

                create table if not exists users (
                  id            integer primary key,
                  email         text not null unique,
                  created_at    integer not null,
                  last_login_at integer,
                  is_admin      integer not null default 0
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

                insert or ignore into plans (code, name, price_cents, download_limit, trial_days, limit_window)
                values ('free', 'Free', 0, 4, 0, 'month');
                "#,
            )
            .unwrap();
            Ok(())
        })
        .unwrap();

        db
    }

    fn req_post_form(path: &str, body: &str) -> Request {
        let req = HttpRequest::builder()
            .method(Method::POST)
            .uri(path)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(())
            .unwrap();

        req.map(|_| Body::from(body.as_bytes().to_vec()))
    }

    fn req_get(path: &str) -> Request {
        let req = HttpRequest::builder()
            .method(Method::GET)
            .uri(path)
            .body(())
            .unwrap();

        req.map(|_| Body::from(Vec::<u8>::new()))
    }

    #[test]
    fn post_request_link_creates_user_and_magic_link() {
        let db = make_db_with_schema();

        let req = req_post_form("/auth/request-link", "email=Test%40Example.com");
        let resp = handle(req, &db).unwrap();

        assert_eq!(resp.status(), 200);

        db.with_conn(|conn| {
            let user_count: i64 = conn
                .query_row(
                    "select count(*) from users where email = 'test@example.com'",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(user_count, 1);

            let ml_count: i64 = conn
                .query_row(
                    r#"select count(*) from magic_links
                       where user_id = (select id from users where email = 'test@example.com')"#,
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(ml_count, 1);

            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn get_magic_consumes_link_and_redirects() {
        let db = make_db_with_schema();

        // Issue a token directly (raw tokens are not stored in DB)
        let token = db
            .with_conn(|conn| {
                let svc = MagicLinkService::new(MagicLinkConfig::default());
                let issued = svc.request_link(conn, "c@d.com", now_unix())?;
                Ok(issued.token)
            })
            .unwrap();

        let req = req_get(&format!("/auth/magic?token={}", token));
        let resp = handle(req, &db).unwrap();

        assert_eq!(resp.status(), 302);

        let loc = resp
            .headers()
            .get("Location")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert_eq!(loc, "/dashboard");

        db.with_conn(|conn| {
            let used_count: i64 = conn
                .query_row(
                    "select count(*) from magic_links where used_at is not null",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert!(used_count >= 1);

            let last_login_set: i64 = conn
                .query_row(
                    "select count(*) from users where last_login_at is not null",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert!(last_login_set >= 1);

            Ok(())
        })
        .unwrap();
    }
}
