use rand::Rng;
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::Duration;

use super::ScraperError;

// Fetch HTML via Zen Rows proxy.
//    ↓
// Is this a block page?
//    ↓
// Parse DOM
//    ↓
// Extract __NEXT_DATA__
//    ↓
// Parse JSON
//    ↓
// Extract properties

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

    pub fn fetch_all_properties_paginated(
        &self,
        base_url: &str,
    ) -> Result<PaginatedResult, ScraperError> {
        let mut all_properties = Vec::new();
        let mut seen_pages = HashSet::new();

        let mut page = 1;
        let mut consecutive_failures = 0;

        const MAX_CONSECUTIVE_FAILURES: usize = 3;
        const MAX_PAGES: usize = 500; // safety cap

        loop {
            if page > MAX_PAGES {
                eprintln!("Reached max page limit ({MAX_PAGES}), stopping");
                break;
            }

            let page_url = if page == 1 {
                base_url.to_string()
            } else {
                format!("{base_url}/pg-{page}")
            };

            eprintln!("📄 Fetching page {page}: {page_url}");

            match self.fetch_properties_via_zenrows(&page_url) {
                Ok(properties) => {
                    consecutive_failures = 0;

                    if properties.is_empty() {
                        eprintln!("No properties found on page {page}, assuming end");
                        break;
                    }

                    // Prevent infinite loops if Realtor serves duplicate pages
                    if !seen_pages.insert(page) {
                        eprintln!("Page {page} already seen, stopping");
                        break;
                    }

                    eprintln!("✅ Page {} fetched ({} properties)", page, properties.len());

                    all_properties.extend(properties);
                    page += 1;

                    // Gentle pacing to reduce block risk
                    std::thread::sleep(Duration::from_secs(2));
                }

                Err(e) => {
                    eprintln!("❌ Page {page} failed: {e}");
                    consecutive_failures += 1;

                    match e {
                        ScraperError::Network(_) | ScraperError::Blocked(_) => {
                            if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
                                eprintln!(
                                        "Too many consecutive failures ({consecutive_failures}), stopping pagination"
                                    );
                                break;
                            }

                            // backoff before retrying same page
                            std::thread::sleep(Duration::from_secs(5));
                            continue;
                        }

                        // Parsing or schema issues should fail fast
                        _ => return Err(e),
                    }
                }
            }
        }

        Ok(PaginatedResult {
            pages_fetched: seen_pages.len(),
            properties: all_properties,
        })
    }

    pub fn fetch_html_via_zenrows(&self, url: &str) -> Result<String, ScraperError> {
        const MAX_ATTEMPTS: u64 = 5;
        const MAX_BACKOFF_SECS: u64 = 10;
        const JITTER_MAX_SECS: u64 = 2;

        let mut last_err = None;

        for attempt in 1..=MAX_ATTEMPTS {
            match self.try_fetch_html_via_zenrows(url) {
                Ok(html) => {
                    eprintln!("ZenRows attempt {attempt} succeeded.");
                    return Ok(html);
                }

                Err(e) => {
                    eprintln!("ZenRows attempt {attempt} failed: {e}");
                    match &e {
                        // ✅ retryable errors
                        ScraperError::Network(_) | ScraperError::Blocked(_) => {
                            last_err = Some(e);
                            // capped backoff
                            let base = std::cmp::min(2 * attempt, MAX_BACKOFF_SECS);
                            let jitter = rand::thread_rng().gen_range(0..=JITTER_MAX_SECS);
                            let delay = base + jitter;

                            eprintln!("Retrying in {}s (base={}, jitter={})", delay, base, jitter);
                            std::thread::sleep(std::time::Duration::from_secs(delay));
                            continue;
                        }
                        // ❌ non-retryable → fail fast
                        _ => return Err(e),
                    }
                }
            }
        }

        Err(last_err.unwrap_or_else(|| ScraperError::Network("ZenRows retry loop failed".into())))
    }

    // Just handles HTML.
    pub fn try_fetch_html_via_zenrows(&self, url: &str) -> Result<String, ScraperError> {
        let api_key = "e10b59e68b56271130e8a20721d14f14457806ae";

        // let api_key = std::env::var("ZENROWS_API_KEY")
        //       .map_err(|_| ScraperError::Config("ZENROWS_API_KEY not set".into()))?;

        use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, REFERER};

        let mut headers = HeaderMap::new();
        headers.insert(REFERER, HeaderValue::from_static("https://www.google.com/"));
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("text/html,application/xhtml+xml"),
        );
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));
        // headers.insert(
        //     USER_AGENT,
        //     HeaderValue::from_static(
        //         "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
        //          AppleWebKit/537.36 (KHTML, like Gecko) \
        //          Chrome/121.0.0.0 Safari/537.36",
        //     ),
        // );

        let mut params = HashMap::new();
        params.insert("url", url);
        params.insert("apikey", &api_key);
        params.insert("js_render", "true");
        params.insert("premium_proxy", "true");
        params.insert("proxy_country", "us");
        params.insert("wait", "2000");

        // 1️⃣ Send request
        let resp = self
            .client
            .get("https://api.zenrows.com/v1/")
            .headers(headers)
            .query(&params)
            .send()
            .map_err(|e| ScraperError::Network(e.to_string()))?;

        // 2️⃣ Check HTTP status
        let status = resp.status();

        // 3️⃣ Read body
        let text = resp
            .text()
            .map_err(|e| ScraperError::Network(e.to_string()))?;

        // 4️⃣ ZenRows HTTP-level error
        if !status.is_success() {
            return Err(ScraperError::Network(format!(
                "ZenRows HTTP {}: {}",
                status, text
            )));
        }

        // 5️⃣ ZenRows API-level error (JSON error body)
        if text.starts_with('{') && text.contains("\"code\":\"RESP") {
            return Err(ScraperError::Network(format!(
                "ZenRows API error: {}",
                text
            )));
        }

        // 6️⃣ Success → HTML
        Ok(text)
    }

    fn extract_next_data(html: &str) -> Result<serde_json::Value, ScraperError> {
        let document = Html::parse_document(html);

        let selector = Selector::parse(r#"script[id="__NEXT_DATA__"]"#)
            .map_err(|e| ScraperError::HtmlParse(e.to_string()))?;

        //Q: & ? .next() ?
        let element = document
            .select(&selector)
            .next()
            .ok_or(ScraperError::MissingNextData)?;

        let json_text = element.text().next().ok_or(ScraperError::MissingNextData)?;

        // Q: ":"
        let data: serde_json::Value =
            serde_json::from_str(json_text).map_err(|e| ScraperError::JsonParse(e.to_string()))?;

        //Q: Ok ?
        Ok(data)
    }

    // Q: -> ?, ok_or_else vs ok_or
    fn extract_properties(
        data: &serde_json::Value,
    ) -> Result<Vec<serde_json::Value>, ScraperError> {
        let properties = data["props"]["pageProps"]["properties"]
            .as_array()
            .ok_or_else(|| {
                ScraperError::UnexpectedShape(
                    "props.pageProps.properties missing or not array".into(),
                )
            })?;

        Ok(properties.clone())
    }

    pub fn fetch_properties_via_zenrows(
        &self,
        url: &str,
    ) -> Result<Vec<serde_json::Value>, ScraperError> {
        let html = self.fetch_html_via_zenrows(url)?;

        //Q: #[] meaning?, & meaning?
        #[cfg(debug_assertions)]
        {
            std::fs::write("realtor_debug.html", &html)
                .map_err(|e| ScraperError::IoError(e.to_string()))?;
        }

        //Q: self?
        let data = Self::extract_next_data(&html)?;
        let properties = Self::extract_properties(&data)?;

        Ok(properties)
    }

    // Below was blocked by Kasada.
    // pub fn fetch_properties(&self, url: &str) -> Result<Vec<Value>, ScraperError> {
    //     let html = self
    //         .client
    //         .get(url)
    //         .send()
    //         .map_err(|e| ScraperError::Network(e.to_string()))?
    //         .text()
    //         .map_err(|e| ScraperError::Network(e.to_string()))?;

    //     // ===============================
    //     // TEMP DEBUG: dump raw HTML
    //     // ===============================
    //     #[cfg(debug_assertions)]
    //     {
    //         std::fs::write("realtor_debug.html", &html)
    //             .map_err(|e| ScraperError::IoError(e.to_string()))?;
    //     }

    //     // ===============================
    //     // BLOCK / BOT PROTECTION CHECK
    //     // ===============================
    //     if html.contains("Your request could not be processed")
    //         || html.contains("unblockrequest@realtor.com")
    //         || html.contains("KPSDK")
    //         || html.contains("Access Denied")
    //         || html.contains("captcha")
    //     {
    //         return Err(ScraperError::Blocked(
    //             "Bot protection / challenge page returned".into(),
    //         ));
    //     }

    //     // ===============================
    //     // NORMAL PARSING FLOW
    //     // ===============================
    //     let document = Html::parse_document(&html);

    //     let selector = Selector::parse(r#"script[id="__NEXT_DATA__"]"#)
    //         .map_err(|e| ScraperError::HtmlParse(e.to_string()))?;

    //     let element = document
    //         .select(&selector)
    //         .next()
    //         .ok_or(ScraperError::MissingNextData)?;

    //     let json_text = element.text().next().ok_or(ScraperError::MissingNextData)?;

    //     let data: Value =
    //         serde_json::from_str(json_text).map_err(|e| ScraperError::JsonParse(e.to_string()))?;

    //     let properties = data["props"]["pageProps"]["properties"]
    //         .as_array()
    //         .ok_or_else(|| {
    //             ScraperError::UnexpectedShape(
    //                 "props.pageProps.properties missing or not array".into(),
    //             )
    //         })?;

    //     Ok(properties.clone())
    // }
}
