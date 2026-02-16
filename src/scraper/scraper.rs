// scraper.rs
use crate::db::connection::Database;
use crate::db::listings::save_properties;
use crate::scraper::Property;
use crate::scraper::ScraperError;
use rand::Rng;
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
// use std::sync::Arc;
use std::time::Duration;

const USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0 Safari/537.36";

pub struct RealtorScraper {
    client: Client,
}

pub struct PaginatedResult {
    pub properties: Vec<Value>,
    pub pages_fetched: usize,
}

impl RealtorScraper {
    pub fn new() -> Result<Self, ScraperError> {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(360))
            .build()
            .map_err(|e| ScraperError::Network(e.to_string()))?;

        Ok(Self { client })
    }

    pub fn run_realtor_scrape(db: &Database, state_name: String, state_abbr: String) {
        let db = db.clone(); // cheap clone (path only)

        std::thread::spawn(move || {
            let now_start = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;

            // Record start
            let run_id = db
                .with_conn(|conn| {
                    crate::db::scrapes::start_scrape_run(conn, &state_abbr, now_start)
                })
                .unwrap_or(0);

            eprintln!("🧵 Scraper thread started for {}", state_name);

            let scraper = match RealtorScraper::new() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Scraper init failed: {e}");
                    // Could update DB here with error
                    return;
                }
            };

            let base_url = format!(
                "https://www.realtor.com/realestateandhomes-search/{}",
                state_name
            );

            let mut total_props = 0;
            let mut pages = 0;

            let result = scraper.fetch_all_properties_paginated(&base_url, |properties| {
                total_props += properties.len();
                pages += 1;
                // 🧠 DB LOGIC LIVES HERE
                save_properties(&db, &properties, &base_url)
                    .map_err(|e| ScraperError::Network(e.to_string()))?;
                Ok(())
            });

            let now_end = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;

            if let Err(e) = result {
                eprintln!("Scrape failed: {e}");
                let _ = db.with_conn(|conn| {
                    crate::db::scrapes::end_scrape_run(
                        conn,
                        run_id,
                        now_end,
                        pages,
                        total_props,
                        false,
                        Some(e.to_string()),
                    )
                });
            } else {
                eprintln!("✅ Scrape complete");
                let _ = db.with_conn(|conn| {
                    crate::db::scrapes::end_scrape_run(
                        conn,
                        run_id,
                        now_end,
                        pages,
                        total_props,
                        true,
                        None,
                    )
                });
            }
        });
    }

    pub fn fetch_all_properties_paginated<F>(
        &self,
        base_url: &str,
        mut on_page: F,
    ) -> Result<(), ScraperError>
    where
        F: FnMut(Vec<Property>) -> Result<(), ScraperError>, // <- updated
    {
        let mut page = 1;
        let mut consecutive_failures = 0;
        let mut seen_pages = HashSet::new();

        loop {
            let page_url = if page == 1 {
                base_url.to_string()
            } else {
                format!("{base_url}/pg-{page}")
            };

            eprintln!("📄 Scraping page {page}: {page_url}");

            match self.fetch_properties_via_zenrows(&page_url) {
                Ok(properties) => {
                    if properties.is_empty() {
                        eprintln!("🏁 No properties found, stopping");
                        break;
                    }

                    if !seen_pages.insert(page) {
                        eprintln!("🔁 Page {page} already seen, stopping");
                        break;
                    }

                    eprintln!("✅ Page {page} parsed ({} properties)", properties.len());

                    on_page(properties)?; // now matches Vec<Property>

                    page += 1;
                    consecutive_failures = 0;
                    std::thread::sleep(Duration::from_secs(2));
                }

                Err(e) => {
                    consecutive_failures += 1;
                    eprintln!("⚠️ Page {page} failed (attempt {consecutive_failures}): {e}");

                    if consecutive_failures >= 3 {
                        eprintln!("❌ Too many failures, aborting scrape");
                        break;
                    }

                    std::thread::sleep(Duration::from_secs(2));
                }
            }
        }

        Ok(())
    }

    pub fn fetch_properties_via_zenrows(&self, url: &str) -> Result<Vec<Property>, ScraperError> {
        let html = self.fetch_html_via_zenrows(url)?;

        #[cfg(debug_assertions)]
        {
            std::fs::write("realtor_debug.html", &html)
                .map_err(|e| ScraperError::IoError(e.to_string()))?;
        }

        let data = Self::extract_next_data(&html)?;
        let properties = Self::extract_properties(&data)?; // returns Vec<Property>

        Ok(properties)
    }

    pub fn fetch_html_via_zenrows(&self, url: &str) -> Result<String, ScraperError> {
        const MAX_ATTEMPTS: u64 = 5;
        const MAX_BACKOFF_SECS: u64 = 10;
        const JITTER_MAX_SECS: u64 = 2;

        let mut last_err = None;

        for attempt in 1..=MAX_ATTEMPTS {
            let start = std::time::Instant::now();

            match self.try_fetch_html_via_zenrows(url) {
                Ok(html) => {
                    eprintln!(
                        "✅ ZenRows success attempt {attempt} in {:?}",
                        start.elapsed()
                    );
                    return Ok(html);
                }
                Err(e) => {
                    eprintln!(
                        "⚠️ ZenRows attempt {attempt} failed in {:?}: {e}",
                        start.elapsed()
                    );

                    last_err = Some(e);

                    // backoff
                    let base = std::cmp::min(2 * attempt, MAX_BACKOFF_SECS);
                    let jitter = rand::thread_rng().gen_range(0..=JITTER_MAX_SECS);
                    std::thread::sleep(Duration::from_secs(base + jitter));
                }
            }
        }

        Err(last_err.unwrap_or_else(|| ScraperError::Network("ZenRows retry loop failed".into())))
    }

    pub fn try_fetch_html_via_zenrows(&self, url: &str) -> Result<String, ScraperError> {
        use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, REFERER};

        let api_key = std::env::var("ZENROWS_API_KEY").map_err(|_| {
            ScraperError::Config("ZENROWS_API_KEY environment variable not set".into())
        })?;

        let mut headers = HeaderMap::new();
        headers.insert(REFERER, HeaderValue::from_static("https://www.google.com/"));
        // headers.insert(
        //     ACCEPT,
        //     HeaderValue::from_static("text/html,application/xhtml+xml"),
        // );
        // headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));

        let mut params = HashMap::new();
        //params.insert("custom_headers", "true".to_string());
        params.insert("url", url.to_string());
        params.insert("apikey", api_key);
        // params.insert("js_render", "true".to_string());
        // params.insert("premium_proxy", "true".to_string());
        // params.insert("proxy_country", "us".to_string());
        // params.insert("wait_for", "script#__NEXT_DATA__".to_string());
        params.insert("original_status", "true".to_string());
        params.insert("mode", "auto".to_string());

        let resp = self
            .client
            .get("https://api.zenrows.com/v1/")
            .headers(headers)
            .query(&params)
            .send()
            .map_err(|e| ScraperError::Network(e.to_string()))?;

        // 1️⃣ ZenRows HTTP status
        let status = resp.status();

        // 2️⃣ ORIGINAL STATUS (add THIS BLOCK)
        let original_status = resp
            .headers()
            .iter()
            .find(|(k, _)| k.as_str().to_ascii_lowercase().contains("original"))
            .map(|(_, v)| v.to_str().unwrap_or("?").to_string())
            .unwrap_or("<none>".to_string());

        // 3️⃣ Now read the body
        let text = resp
            .text()
            .map_err(|e| ScraperError::Network(e.to_string()))?;

        if !status.is_success() {
            return Err(ScraperError::Network(format!(
                "ZenRows HTTP {} ({}) : {}",
                status, original_status, text
            )));
        }

        if text.starts_with('{') {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                if json.get("code").is_some() {
                    return Err(ScraperError::Network(format!(
                        "ZenRows API error ({}) : {}",
                        original_status, text
                    )));
                }
            }
        }

        Ok(text)
    }

    fn extract_next_data(html: &str) -> Result<Value, ScraperError> {
        let document = Html::parse_document(html);
        let selector = Selector::parse(r#"script[id="__NEXT_DATA__"]"#)
            .map_err(|e| ScraperError::HtmlParse(e.to_string()))?;

        let element = document
            .select(&selector)
            .next()
            .ok_or(ScraperError::MissingNextData)?;

        let json_text = element.text().next().ok_or(ScraperError::MissingNextData)?;
        let data: Value =
            serde_json::from_str(json_text).map_err(|e| ScraperError::JsonParse(e.to_string()))?;
        Ok(data)
    }

    fn extract_properties(data: &Value) -> Result<Vec<Property>, ScraperError> {
        let arr = data["props"]["pageProps"]["properties"].as_array().ok_or(
            ScraperError::UnexpectedShape("properties missing".to_string()),
        )?;

        let properties: Result<Vec<_>, _> = arr
            .iter()
            .map(|v| serde_json::from_value(v.clone()))
            .collect();

        properties.map_err(|e| ScraperError::Deserialize(e.to_string()))
    }
}
