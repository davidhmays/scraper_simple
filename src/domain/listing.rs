#[derive(Debug)]
pub struct ListingWithProperty {
    // IDs (TEMP: mirror source IDs)
    pub property_id: String,
    pub listing_id: String,

    // Source identity
    pub source: String,            // MLS name
    pub source_id: String,         // ID of MLS system itself
    pub source_listing_id: String, // Listing id within an MLS
    pub source_property_id: String,

    pub address_line: String,
    pub city: String,
    pub state_abbr: String,
    pub postal_code: Option<String>,
    pub county_name: Option<String>,

    // pub type:
    pub bedrooms: Option<i64>,
    pub bathrooms: Option<i64>,

    pub list_price: i64,
    pub status: String,

    pub is_coming_soon: bool,
    pub is_contingent: bool,
    pub is_pending: bool,
}
