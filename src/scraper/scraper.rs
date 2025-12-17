use reqwest::blocking::Client;
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::HashMap;
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

impl RealtorScraper {
    pub fn new() -> Result<Self, ScraperError> {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(180))
            .build()
            .map_err(|e| ScraperError::Network(e.to_string()))?;

        Ok(Self { client })
    }

    pub fn fetch_via_zenrows(&self, url: &str) -> Result<String, ScraperError> {
        //TODO: API KEY AS ENV VARIABLE
        let api_key = _________

        // let api_key = std::env::var("ZENROWS_API_KEY")
        //     .map_err(|_| ScraperError::Config("ZENROWS_API_KEY not set".into()))?;

        let mut params = HashMap::new();
        params.insert("url", url);
        params.insert("apikey", &api_key);
        params.insert("js_render", "true");
        params.insert("premium_proxy", "true");
        // params.insert("autoparse", "true");

        let resp = self
            .client
            .get("https://api.zenrows.com/v1/")
            .query(&params)
            .send()
            .map_err(|e| ScraperError::Network(e.to_string()))?
            .text()
            .map_err(|e| ScraperError::Network(e.to_string()))?;

        Ok(resp)
    }

    pub fn fetch_properties(&self, url: &str) -> Result<Vec<Value>, ScraperError> {
        let html = self
            .client
            .get(url)
            .send()
            .map_err(|e| ScraperError::Network(e.to_string()))?
            .text()
            .map_err(|e| ScraperError::Network(e.to_string()))?;

        // ===============================
        // TEMP DEBUG: dump raw HTML
        // ===============================
        #[cfg(debug_assertions)]
        {
            std::fs::write("realtor_debug.html", &html)
                .map_err(|e| ScraperError::IoError(e.to_string()))?;
        }

        // ===============================
        // BLOCK / BOT PROTECTION CHECK
        // ===============================
        if html.contains("Your request could not be processed")
            || html.contains("unblockrequest@realtor.com")
            || html.contains("KPSDK")
            || html.contains("Access Denied")
            || html.contains("captcha")
        {
            return Err(ScraperError::Blocked(
                "Bot protection / challenge page returned".into(),
            ));
        }

        // ===============================
        // NORMAL PARSING FLOW
        // ===============================
        let document = Html::parse_document(&html);

        let selector = Selector::parse(r#"script[id="__NEXT_DATA__"]"#)
            .map_err(|e| ScraperError::HtmlParse(e.to_string()))?;

        let element = document
            .select(&selector)
            .next()
            .ok_or(ScraperError::MissingNextData)?;

        let json_text = element.text().next().ok_or(ScraperError::MissingNextData)?;

        let data: Value =
            serde_json::from_str(json_text).map_err(|e| ScraperError::JsonParse(e.to_string()))?;

        let properties = data["props"]["pageProps"]["properties"]
            .as_array()
            .ok_or_else(|| {
                ScraperError::UnexpectedShape(
                    "props.pageProps.properties missing or not array".into(),
                )
            })?;

        Ok(properties.clone())
    }
}
