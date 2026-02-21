// src/domain/logic.rs

use chrono::NaiveDateTime;

/// Determines the canonical status of a property based on a set of business rules.
/// The order of checks determines the precedence of the status lifecycle.
///
/// For example, a property can be both 'pending' and 'contingent', but 'pending'
/// takes precedence in our lifecycle model.
pub fn derive_canonical_status(
    sold_date: &Option<NaiveDateTime>,
    is_pending: bool,
    is_contingent: bool,
    // Note: This logic assumes 'is_coming_soon' is being tracked.
    // We will need to add it to the data pipeline if it's not already.
    is_coming_soon: bool,
    raw_status: &Option<String>,
) -> &'static str {
    if sold_date.is_some() {
        return "Sold";
    }
    if is_pending {
        return "Pending";
    }
    if is_contingent {
        return "Contingent";
    }
    if is_coming_soon {
        return "Coming Soon";
    }
    if let Some(status) = raw_status {
        match status.as_str() {
            // These are considered our base "active" statuses.
            "for_sale" | "ready_to_build" | "for_rent" => "Active",
            // Any other raw status from the scraper that isn't overridden
            // by a higher-priority flag will be categorized as 'Other'.
            _ => "Other",
        }
    } else {
        // If we don't even have a raw status, it's definitely 'Other'.
        "Other"
    }
}
