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
            .timeout(Duration::from_secs(360))
            .build()
            .map_err(|e| ScraperError::Network(e.to_string()))?;

        Ok(Self { client })
    }

    // Just handles HTML.
    pub fn fetch_html_via_zenrows(&self, url: &str) -> Result<String, ScraperError> {
        //TODO: API KEY AS ENV VARIABLE
        let api_key = ___

        // let api_key = std::env::var("ZENROWS_API_KEY")
        //     .map_err(|_| ScraperError::Config("ZENROWS_API_KEY not set".into()))?;

        use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, REFERER};

        //Q: rotate headers?
        let mut headers = HeaderMap::new();
        headers.insert(REFERER, HeaderValue::from_static("https://www.google.com/"));
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("text/html,application/xhtml+xml"),
        );
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));

        let mut params = HashMap::new();
        params.insert("url", url);
        params.insert("apikey", &api_key);
        params.insert("js_render", "true");
        params.insert("premium_proxy", "true");
        params.insert("proxy_country", "us");
        params.insert("wait", "8000");
        // params.insert("autoparse", "true");
        // wait_for if JS is flaky
        //retry=true (ZenRows feature)

        let resp = self
            .client
            .get("https://api.zenrows.com/v1/")
            .headers(headers)
            .query(&params)
            .send()
            .map_err(|e| ScraperError::Network(e.to_string()))?
            .text()
            .map_err(|e| ScraperError::Network(e.to_string()))?;

        Ok(resp)
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
