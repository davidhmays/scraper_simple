use serde::Deserialize;

// prop
//  ├── source
//  │    ├── name
//  │    ├── id
//  │    └── listing_id
//  ├── location
//  │    ├── address
//  │    │    ├── line
//  │    │    ├── city
//  │    │    ├── state_code
//  │    │    ├── postal_code
//  │    │    └── country
//  │    ├── county
//  │    │    ├── name
//  │    │    └── fips_code
//  │    └── coordinate
//  │         ├── lat
//  │         └── lon
//  └── description
//       ├── beds
//       ├── baths
//       ├── lot_sqft
//       └── type

#[derive(Debug, Deserialize)]
pub struct Property {
    pub source: Source,
    pub location: Location,
    pub description: Description,

    pub status: Option<String>,
    pub list_price: Option<i64>,
    pub price_reduced: Option<i64>,
    pub sold_price: Option<i64>,

    pub flags: Option<Flags>,
}

#[derive(Debug, Deserialize)]
pub struct Source {
    pub name: Option<String>,
    pub id: Option<String>,
    #[serde(rename = "listing_id")]
    pub listing_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Location {
    pub address: Option<Address>,
    pub county: Option<County>,
    pub coordinate: Option<Coordinate>,
}

#[derive(Debug, Deserialize)]
pub struct Address {
    pub line: Option<String>,
    pub city: Option<String>,
    #[serde(rename = "state_code")]
    pub state_code: Option<String>,
    #[serde(rename = "postal_code")]
    pub postal_code: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct County {
    pub name: Option<String>,
    #[serde(rename = "fips_code")]
    pub fips_code: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct Coordinate {
    pub lat: Option<f64>,
    pub lon: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct Description {
    pub beds: Option<i64>,
    pub baths: Option<i64>,
    #[serde(rename = "lot_sqft")]
    pub lot_sqft: Option<i64>,
    #[serde(rename = "type")]
    pub property_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Flags {
    pub is_coming_soon: Option<bool>,
    pub is_contingent: Option<bool>,
    pub is_foreclosure: Option<bool>,
    pub is_new_construction: Option<bool>,
    pub is_new_listing: Option<bool>,
    pub is_pending: Option<bool>,

    // "is_price_reduced" appears both in flags and top-level JSON, optional
    pub is_price_reduced: Option<bool>,
}
