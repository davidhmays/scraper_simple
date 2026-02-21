// src/domain/changes.rs

use chrono::NaiveDateTime;

/// A ViewModel representing a single change event for a property.
/// This is the definitive structure for both the UI preview and the spreadsheet export,
/// designed to be easily filterable.
#[derive(Debug)]
pub struct ChangeViewModel {
    // === Event Details ===
    pub change_date: NaiveDateTime,
    /// The primary type of change, simplified for the user (e.g., "Status Change", "Price Change").
    pub change_type: String,
    /// The value of the field before the change. For a status change, this will be the canonical status.
    pub previous_value: String,
    /// The value of the field after the change. For a status change, this will be the canonical status.
    pub current_value: String,

    // === Property Context (current state at time of change) ===
    // Address
    pub address_full: String,
    pub address_line: String,
    pub city: String,
    pub county_name: Option<String>,
    pub state_abbr: Option<String>,
    pub postal_code: String,
    // Details
    pub price: Option<i64>,       // The current price for context
    pub canonical_status: String, // The derived lifecycle status at the time of change
    // Flags
    pub is_ready_to_build: bool,
    pub is_new_listing: bool,
    pub is_price_reduced: bool,
    pub is_foreclosure: bool,

    // === Calculated Deltas ===
    /// The amount of a price reduction, if applicable.
    pub price_reduction: Option<i64>,
}
