pub mod models;
mod scraper;
mod scraper_error;

pub use models::Property;
pub use scraper::RealtorScraper;
pub use scraper_error::ScraperError;
