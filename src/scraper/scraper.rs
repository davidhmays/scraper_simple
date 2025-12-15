use reqwest::blocking::Client;
use scraper::{Html, Selector};
use serde_json::Value;

use super::ScraperError;

const USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0 Safari/537.36";

pub struct RealtorScraper {
    client: Client,
}

impl RealtorScraper {
    pub fn new() -> Result<Self, ScraperError> {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .map_err(|e| ScraperError::Network(e.to_string()))?;

        Ok(Self { client })
    }

    pub fn fetch_properties(&self, url: &str) -> Result<Vec<Value>, ScraperError> {
        let html = self
            .client
            .get(url)
            .send()
            .map_err(|e| ScraperError::Network(e.to_string()))?
            .text()
            .map_err(|e| ScraperError::Network(e.to_string()))?;

        if html.contains("Access Denied") || html.contains("captcha") {
            return Err(ScraperError::Blocked("Access denied page".into()));
        }

        let document = Html::parse_document(&html);
        let selector = Selector::parse(r#"script#__NEXT_DATA__"#)
            .map_err(|e| ScraperError::HtmlParse(e.to_string()))?;

        let element = document
            .select(&selector)
            .next()
            .ok_or(ScraperError::MissingNextData)?;

        let json_text = element.inner_html();

        let data: Value =
            serde_json::from_str(&json_text).map_err(|e| ScraperError::JsonParse(e.to_string()))?;

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
