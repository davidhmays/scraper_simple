// src/domain/property.rs

use crate::scraper::models::Property as ScraperProperty;
use chrono::{DateTime, NaiveDateTime, Utc};

/// Represents a property as scraped, flattened, and normalized, ready for comparison.
/// This acts as an anti-corruption layer between the raw scrape and our database models.
#[derive(Debug, PartialEq, Clone)]
pub struct ScrapedProperty {
    // Source identifier
    pub source_name: String,
    pub source_listing_id: String,

    // Address fields (used for unique identification)
    pub address_line: String,
    pub city: String,
    pub postal_code: String,
    pub state_abbr: Option<String>,
    pub county_name: Option<String>,

    // Tracked fields
    pub status: Option<String>,
    pub list_price: Option<i64>,
    pub sold_price: Option<i64>,
    pub sold_date: Option<NaiveDateTime>,
    pub is_pending: Option<bool>,
    pub is_contingent: Option<bool>,
    pub is_new_listing: Option<bool>,
    pub is_foreclosure: Option<bool>,
    pub is_price_reduced: Option<bool>,
    pub is_coming_soon: Option<bool>,
}

impl ScrapedProperty {
    /// Creates a flattened, clean `ScrapedProperty` from the raw nested scraper model.
    /// It validates that essential fields required for identification exist.
    pub fn from_scraper_property(prop: &ScraperProperty) -> Result<Self, String> {
        let address = prop
            .location
            .address
            .as_ref()
            .ok_or("Missing address object")?;

        let address_line = address
            .line
            .as_deref()
            .filter(|s| !s.is_empty())
            .ok_or("Missing or empty address line")?
            .to_string();

        let city = address
            .city
            .as_deref()
            .filter(|s| !s.is_empty())
            .ok_or("Missing or empty city")?
            .to_string();

        let postal_code = address
            .postal_code
            .as_deref()
            .filter(|s| !s.is_empty())
            .ok_or("Missing or empty postal code")?
            .to_string();

        let source_listing_id = prop
            .source
            .listing_id
            .as_deref()
            .filter(|s| !s.is_empty())
            .ok_or("Missing or empty source listing id")?
            .to_string();

        // Helper to parse optional date strings from the scrape into NaiveDateTime
        let parse_date = |date_str: Option<&str>| {
            date_str
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc).naive_utc())
        };

        let description = prop.description.as_ref();
        let sold_date = parse_date(description.and_then(|d| d.sold_date.as_deref()));

        Ok(ScrapedProperty {
            source_name: prop.source.name.as_deref().unwrap_or("unknown").to_string(),
            source_listing_id,
            address_line,
            city,
            postal_code,
            state_abbr: address.state_code.clone(),
            county_name: prop.location.county.as_ref().and_then(|c| c.name.clone()),
            status: prop.status.clone(),
            list_price: prop.list_price,
            sold_price: prop.sold_price,
            sold_date,
            is_pending: prop.flags.as_ref().and_then(|f| f.is_pending),
            is_contingent: prop.flags.as_ref().and_then(|f| f.is_contingent),
            is_new_listing: prop.flags.as_ref().and_then(|f| f.is_new_listing),
            is_foreclosure: prop.flags.as_ref().and_then(|f| f.is_foreclosure),
            is_price_reduced: prop.flags.as_ref().and_then(|f| f.is_price_reduced),
            is_coming_soon: prop.flags.as_ref().and_then(|f| f.is_coming_soon),
        })
    }
}

/// Represents the current state of a property as stored in our `properties` table.
#[derive(Debug, PartialEq, Clone)]
pub struct TrackedProperty {
    pub id: i64,
    pub status: Option<String>,
    pub list_price: Option<i64>,
    pub sold_price: Option<i64>,
    pub sold_date: Option<NaiveDateTime>,
    pub is_pending: Option<bool>,
    pub is_contingent: Option<bool>,
    pub is_new_listing: Option<bool>,
    pub is_foreclosure: Option<bool>,
    pub is_price_reduced: Option<bool>,
    pub is_coming_soon: Option<bool>,
}

/// Represents a single change to a tracked field, to be stored in `property_history`.
#[derive(Debug)]
pub struct PropertyChange {
    pub property_id: i64,
    pub field_name: String,
    pub previous_value: Option<String>,
    pub current_value: String,
}

impl TrackedProperty {
    /// Compares the current state of a tracked property with a newly scraped version
    /// and generates a list of changes to be logged.
    pub fn diff(&self, new: &ScrapedProperty) -> Vec<PropertyChange> {
        let mut changes = Vec::new();

        // This macro reduces the boilerplate of comparing each field.
        // It handles Option types and converts any value to a String for the history log.
        macro_rules! compare_and_log {
            ($field:ident, $field_name:expr) => {
                if self.$field != new.$field {
                    changes.push(PropertyChange {
                        property_id: self.id,
                        field_name: $field_name.to_string(),
                        previous_value: self.$field.as_ref().map(|v| v.to_string()),
                        current_value: new
                            .$field
                            .as_ref()
                            .map(|v| v.to_string())
                            .unwrap_or_default(),
                    });
                }
            };
        }

        compare_and_log!(status, "status");
        compare_and_log!(list_price, "list_price");
        compare_and_log!(sold_price, "sold_price");
        compare_and_log!(sold_date, "sold_date");
        compare_and_log!(is_pending, "is_pending");
        compare_and_log!(is_contingent, "is_contingent");
        compare_and_log!(is_new_listing, "is_new_listing");
        compare_and_log!(is_foreclosure, "is_foreclosure");
        compare_and_log!(is_price_reduced, "is_price_reduced");
        compare_and_log!(is_coming_soon, "is_coming_soon");

        changes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_property_diff_logic() {
        let before_date = NaiveDate::from_ymd_opt(2023, 10, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();

        // This represents the state of the property as it is in our database.
        let before = TrackedProperty {
            id: 1,
            status: Some("for_sale".to_string()),
            list_price: Some(500000),
            sold_price: None,
            sold_date: None,
            is_pending: Some(false),
            is_contingent: Some(true),
            is_new_listing: Some(true),
            is_foreclosure: Some(false),
            is_price_reduced: Some(false),
            is_coming_soon: Some(false),
        };

        // This represents the new data we just scraped for the same property.
        let after = ScrapedProperty {
            // Address fields are not used in the diff, so they can be dummy values.
            source_name: "test".to_string(),
            source_listing_id: "123".to_string(),
            address_line: "123 Main".to_string(),
            city: "Anytown".to_string(),
            postal_code: "12345".to_string(),
            state_abbr: Some("CA".to_string()),
            county_name: None,

            // --- Define the changes ---
            status: Some("contingent".to_string()), // Changed from "for_sale"
            list_price: Some(495000),               // Changed (price drop)
            sold_price: None,                       // Unchanged
            sold_date: Some(before_date),           // Changed from None
            is_pending: Some(true),                 // Changed from false
            is_contingent: None,                    // Changed from Some(true) to None
            is_new_listing: Some(false),            // Changed from true to false
            is_foreclosure: Some(true),             // Changed from false to true
            is_price_reduced: Some(true),           // Changed from false to true
            is_coming_soon: Some(true),             // Changed from false to true
        };

        // Get the list of changes.
        let changes = before.diff(&after);

        // We expect exactly 9 fields to have changed.
        assert_eq!(changes.len(), 9);

        // Helper to find a specific change in the vector for easier assertions.
        let find_change = |name: &str| {
            changes
                .iter()
                .find(|c| c.field_name == name)
                .expect(&format!("Change for '{}' not found", name))
        };

        // 1. Verify the status change
        let status_change = find_change("status");
        assert_eq!(status_change.previous_value, Some("for_sale".to_string()));
        assert_eq!(status_change.current_value, "contingent".to_string());

        // 2. Verify the price change
        let price_change = find_change("list_price");
        assert_eq!(price_change.previous_value, Some("500000".to_string()));
        assert_eq!(price_change.current_value, "495000".to_string());

        // 3. Verify the sold_date was added
        let sold_date_change = find_change("sold_date");
        assert_eq!(sold_date_change.previous_value, None);
        assert_eq!(sold_date_change.current_value, before_date.to_string());

        // 4. Verify the is_pending flag change
        let pending_change = find_change("is_pending");
        assert_eq!(pending_change.previous_value, Some("false".to_string()));
        assert_eq!(pending_change.current_value, "true".to_string());

        // 5. Verify the is_contingent flag change (from Some to None)
        let contingent_change = find_change("is_contingent");
        assert_eq!(contingent_change.previous_value, Some("true".to_string()));
        assert_eq!(contingent_change.current_value, ""); // None becomes empty string

        let new_listing_change = find_change("is_new_listing");
        assert_eq!(new_listing_change.previous_value, Some("true".to_string()));
        assert_eq!(new_listing_change.current_value, "false".to_string());

        let foreclosure_change = find_change("is_foreclosure");
        assert_eq!(foreclosure_change.previous_value, Some("false".to_string()));
        assert_eq!(foreclosure_change.current_value, "true".to_string());

        let price_reduced_change = find_change("is_price_reduced");
        assert_eq!(
            price_reduced_change.previous_value,
            Some("false".to_string())
        );
        assert_eq!(price_reduced_change.current_value, "true".to_string());

        let coming_soon_change = find_change("is_coming_soon");
        assert_eq!(coming_soon_change.previous_value, Some("false".to_string()));
        assert_eq!(coming_soon_change.current_value, "true".to_string());
    }
}
