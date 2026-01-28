use crate::db::connection::Database;
use crate::db::listings::get_counties_by_state;
use crate::db::listings::get_listings_by_state;

use crate::errors::ServerError;
use crate::mailings::{
    generate_mailings_for_campaign, ListingFlag, MediaType, NewCampaign, PropertyType,
};
use crate::responses::{html_response, xlsx_response, ResultResp};
use crate::scraper::RealtorScraper;
use crate::spreadsheets::{
    export_listings_xlsx, export_mailings_xlsx, get_mailings_export_rows, MailingExportRow,
};

use crate::templates;

use astra::{Body, Request, ResponseBuilder};
use maud::html;
use rust_xlsxwriter::Workbook;
use std::collections::HashMap;
use std::io::Read;
use url::form_urlencoded; // for read_to_end

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

        // WARN: Has some hard-coded values!
        ("POST", "/campaigns") => {
            let body_bytes = body_to_bytes(&mut req)?;
            let pairs: Vec<(String, String)> =
                form_urlencoded::parse(&body_bytes).into_owned().collect();

            eprintln!("POST /campaigns raw form pairs = {:?}", pairs);

            let state = form_first(&pairs, "state")
                .unwrap_or_else(|| "UT".to_string())
                .to_uppercase();

            let county = form_first(&pairs, "county").unwrap_or_default(); // optional, not used yet

            let flag_strs = form_all(&pairs, "flags");
            let any_of_flags: Vec<ListingFlag> =
                flag_strs.iter().filter_map(|s| parse_flag(s)).collect();

            let type_strs = form_all(&pairs, "types");
            let any_of_types: Vec<PropertyType> =
                type_strs.iter().filter_map(|s| parse_type(s)).collect();

            if state.trim().is_empty() {
                return Err(ServerError::BadRequest("state must not be empty".into()));
            }
            if any_of_flags.is_empty() {
                return Err(ServerError::BadRequest("flags must not be empty".into()));
            }
            if any_of_types.is_empty() {
                return Err(ServerError::BadRequest("types must not be empty".into()));
            }

            let campaign_name = if county.trim().is_empty() {
                format!("{} UI Campaign", state)
            } else {
                format!("{} {} UI Campaign", state, county)
            };

            let campaign = NewCampaign {
                name: campaign_name,
                variant: "A".to_string(),
                description: Some("ui-driven campaign".to_string()),
                media_type: MediaType::Postcard,
                media_size: "6x9".to_string(),
                any_of_flags,
                any_of_types,
                state_abbr: state.clone(),
                zip_codes: vec![], // ✅ statewide mode now that campaign.rs allows it
            };

            generate_mailings_for_campaign(db, &campaign)?;

            let rows = get_mailings_export_rows(db, &campaign.name, &campaign.variant)?;
            export_mailings_xlsx(&rows, &format!("campaign_{}_A.xlsx", state))
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
