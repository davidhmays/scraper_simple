use crate::db::connection::Database;
use crate::db::listings::get_counties_by_state;
use crate::db::listings::get_listings_by_state;

use crate::errors::ServerError;
use crate::mailings::{
    generate_mailings_for_campaign, BrevoMailer, ListingFlag, MediaType, NewCampaign, PropertyType,
};
use crate::responses::{html_response, xlsx_response, ResultResp};
use crate::scraper::RealtorScraper;
use crate::spreadsheets::{
    export_listings_xlsx, export_mailings_xlsx, get_mailings_export_rows, MailingExportRow,
};

use crate::templates;
use crate::templates::pages::dashboard::DashboardVm;

use astra::{Body, Request, ResponseBuilder};
use maud::html;
use rust_xlsxwriter::Workbook;
use std::collections::HashMap;
use std::io::Read;
use url::form_urlencoded; // for read_to_end

use crate::auth::sessions;
use crate::db::magic_auth::{redeem_magic_link, request_magic_link};

use std::time::{SystemTime, UNIX_EPOCH};

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
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

    // Option A: use the BodyReader (implements std::io::Read)
    req.body_mut()
        .reader()
        .read_to_end(&mut out)
        .map_err(|e| ServerError::BadRequest(format!("Failed to read request body: {e}")))?;

    Ok(out)

    // Option B (equivalent): iterate over chunks
    // for chunk in req.body_mut() {
    //     let chunk = chunk.map_err(|e| ServerError::BadRequest(format!("Body chunk error: {e}")))?;
    //     out.extend_from_slice(&chunk);
    // }
    // Ok(out)
}

fn parse_flag(s: &str) -> Option<ListingFlag> {
    match s.trim().to_lowercase().as_str() {
        "coming_soon" => Some(ListingFlag::ComingSoon),
        "contingent" => Some(ListingFlag::Contingent),
        "foreclosure" => Some(ListingFlag::Foreclosure),
        "new_construction" => Some(ListingFlag::NewConstruction),
        "new_listing" => Some(ListingFlag::NewListing),
        "pending" => Some(ListingFlag::Pending),
        _ => None,
    }
}

fn parse_type(s: &str) -> Option<PropertyType> {
    match s.trim().to_lowercase().as_str() {
        "single_family" => Some(PropertyType::SingleFamily),
        "townhomes" => Some(PropertyType::Townhomes),
        "land" => Some(PropertyType::Land),
        "multi_family" => Some(PropertyType::MultiFamily),
        "farm" => Some(PropertyType::Farm),
        "condos" => Some(PropertyType::Condos),
        _ => None,
    }
}

//TODO: look at making only post/campaigns mutable.
pub fn handle(mut req: Request, db: &Database) -> ResultResp {
    let method = req.method().as_str();
    let path = req.uri().path();

    match (method, path) {
        ("GET", path) if path.starts_with("/static") => serve_static(path),
        ("GET", "/") => html_response(templates::pages::home_page()),
        ("GET", "/admin") => html_response(templates::pages::admin_page()),
        ("GET", "/campaigns") => {
            // Default state if not provided in query string
            let mut state = "UT".to_string();

            // Optional: support /campaigns?state=UT
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

        // Spawn scraper background job
        ("GET", "/scrape-test") => {
            let db_clone = db.clone(); // Clone the Database for the thread

            // Spawn background thread
            std::thread::spawn(move || {
                eprintln!("🚀 Background scrape job started");
                RealtorScraper::run_realtor_scrape(&db_clone);
            });

            // Immediately return OK response to browser
            let body = html! {
                h1 { "Scraper triggered in background" }
                p { "Check logs for progress." }
            };
            html_response(body)
        }

        //TODO: Should state_abbr be a reference?
        ("GET", path) if path.starts_with("/export/") => {
            let state = path.trim_start_matches("/export/").to_uppercase();

            let listings = get_listings_by_state(db, &state)?;
            export_listings_xlsx(&listings, &state)
        }

        ("GET", "/dashboard") => {
            let now = now_unix();

            // Step 1: get the logged-in user
            let user = current_user(&req, db, now)?;

            let Some((user_id, email)) = user else {
                // Not logged in, redirect to home
                return Ok(ResponseBuilder::new()
                    .status(302)
                    .header("Location", "/")
                    .body(Body::empty())
                    .unwrap());
            };

            // Step 2: fetch dashboard info
            let dashboard_vm = db.with_conn(|conn| {
                let plan_info = crate::db::plans::get_user_plan(conn, user_id)
                    .map_err(|e| ServerError::DbError(e.to_string()))?;

                let is_admin = crate::db::users::is_user_admin(conn, user_id)
                    .map_err(|e| ServerError::DbError(e.to_string()))?;

                Ok(DashboardVm {
                    email,
                    plan_code: plan_info.code,
                    plan_name: plan_info.name,
                    download_limit: plan_info.download_limit,
                    is_admin,
                })
            })?;

            // Step 3: render template
            html_response(templates::pages::dashboard_page(&dashboard_vm))
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

        // WARN: Has some hard-coded values!
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

            let body = html! {
                div class="p-4 rounded border" {
                    h3 { "Check your email" }
                    p { "If that address exists, we sent a sign-in link." }
                    p class="text-sm opacity-70" { "Dev mode: link was logged to server output." }
                }
            };
            html_response(body)
        }

        _ => Err(ServerError::NotFound),
    }
}

fn form_first(pairs: &[(String, String)], key: &str) -> Option<String> {
    pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
}

fn form_all(pairs: &[(String, String)], key: &str) -> Vec<String> {
    pairs
        .iter()
        .filter(|(k, _)| k == key)
        .map(|(_, v)| v.clone())
        .collect()
}

fn parse_zip_codes(raw: &str) -> Vec<String> {
    raw.split(|c: char| c == ',' || c.is_whitespace())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
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
    use crate::db::connection::Database;
    use crate::init_db;
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

        init_db(&db, "sql/schema.sql").expect("Failed to initialize DB");

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
    fn get_magic_consumes_link_and_redirects() -> Result<(), ServerError> {
        let db = make_db_with_schema();

        // Step 1: issue a token via the MagicLinkService
        let token = db.with_conn(|conn| {
            let svc = crate::auth::magic::MagicLinkService::new(
                crate::auth::magic::MagicLinkConfig::default(),
            );
            let issued = svc.request_link(conn, "c@d.com", now_unix())?;
            Ok::<_, ServerError>(issued.token)
        })?;

        // Step 2: hit the /auth/magic endpoint
        let req = req_get(&format!("/auth/magic?token={}", token));
        let resp = handle(req, &db)?;

        assert_eq!(resp.status(), 302);

        let loc = resp
            .headers()
            .get("Location")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert_eq!(loc, "/dashboard");

        // Step 3: verify DB state using production-style error handling
        db.with_conn(|conn| {
            let used_count: i64 = conn
                .query_row(
                    "select count(*) from magic_links where used_at is not null",
                    [],
                    |r| r.get(0),
                )
                .map_err(|e| ServerError::DbError(format!("query magic_links failed: {e}")))?;
            assert!(used_count >= 1);

            let last_login_set: i64 = conn
                .query_row(
                    "select count(*) from users where last_login_at is not null",
                    [],
                    |r| r.get(0),
                )
                .map_err(|e| ServerError::DbError(format!("query users failed: {e}")))?;
            assert!(last_login_set >= 1);

            Ok(())
        })?;

        Ok(())
    }
}
